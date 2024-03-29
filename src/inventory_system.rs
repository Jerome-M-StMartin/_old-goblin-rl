use specs::prelude::*;
use super::{PickUpIntent, Name, InBackpack, Position, gui::gamelog, UseItemIntent, RunState,
            DropItemIntent, Consumable, Healing, Heals, DamageOnUse, DamageQueue, Map, AoE, Confusion,
            particle_system::ParticleBuilder, MagicMapper, Aflame, Equipped, UnequipIntent};
//use bracket_lib::prelude::{console};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteStorage<'a, PickUpIntent>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, Aflame>,
                        ReadStorage<'a, Name>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut pickup_intents, mut positions, mut in_backpack,
             mut aflame_storage, names) = data;

        let mut logger = gamelog::Logger::new();
             
        for intent in pickup_intents.join() {
            //If item is not already picked-up...(Is Some if item has position, else None).
            if let Some(_) = positions.remove(intent.item) {
                in_backpack.insert(intent.item, InBackpack { owner: intent.desired_by })
                    .expect("Unable to insert item into backpack.");
               
                //snuf aflame items as it enters backpack
                if let Some(_) = aflame_storage.get(intent.item) {
                    aflame_storage.remove(intent.item);
                }

                if intent.desired_by == *player_entity {
                    logger.append(format!("{} placed into inventory.", 
                            names.get(intent.item).unwrap().name));
                }
            }
        }

        pickup_intents.clear();
        logger.log();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, RunState>,
                        WriteExpect<'a, ParticleBuilder>,
                        ReadExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, UseItemIntent>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Consumable>,
                        WriteStorage<'a, Healing>,
                        ReadStorage<'a, Heals>,
                        ReadStorage<'a, DamageOnUse>,
                        WriteStorage<'a, DamageQueue>,
                        ReadStorage<'a, AoE>,
                        WriteStorage<'a, Confusion>,
                        ReadStorage<'a, MagicMapper>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut runstate, mut particle_builder, map, entities, mut use_item_intent,
             names, consumables, mut healing_storage, heals_storage, inflicts_damage, mut damage_queue,
             aoe, mut confusion,  magic_mapper) = data;

        let mut logger = gamelog::Logger::new();

        for (entity, use_intent) in (&entities, &use_item_intent).join() {
            let mut is_item_used = true;
            let mut targets: Vec<Entity> = Vec::new();
           
            //targeting logic
            match use_intent.target { 
                None => {targets.push(*player_entity);} //Assume Player is target.
                Some(target) => {
                    let area_effect = aoe.get(use_intent.item);
                    match area_effect {
                        None => { //Single target
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => { //AoE
                            let mut blast_tiles = bracket_lib::prelude::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 &&
                                                   p.x < map.width - 1 &&
                                                   p.y > 0 &&
                                                   p.y < map.height - 1);
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }

                                //spawn particles
                                particle_builder.request(tile_idx.x, tile_idx.y,
                                    bracket_lib::prelude::RGB::named(bracket_lib::prelude::ORANGE), bracket_lib::prelude::RGB::named(bracket_lib::prelude::BLACK),
                                    bracket_lib::prelude::to_cp437('░'), 200.0);
                            }
                        }
                            
                    }
                }
            }

            /*//If item is equippable, insert EquipIntent or UnequipIntent.
            //Further logic for this item is to be handled by EquipSystem.
            if let Some(_e) = equippable.get(use_intent.item) {//if equippable
                if let Some(_e) = equipped.get(use_intent.item) {//if already equipped
                    unequip_intent.insert(entity, UnequipIntent {item: use_intent.item})
                        .expect("Failed to insert UnequipIntent component.");
                } else {//if not yet equipped
                    equip_intent.insert(entity, EquipIntent {item: use_intent.item})
                        .expect("Failed to insert EquipIntent component.");
                }
            }*/

            //damaging item logic
            let damaging_item = inflicts_damage.get(use_intent.item);
            match damaging_item {
                None => {}
                Some(d) => {
                    is_item_used = false;
                    
                    for mob in targets.iter() {
                        for dmg_atom in d.dmg_atoms.iter() {
                            DamageQueue::queue_damage(&mut damage_queue, *mob, *dmg_atom);
                        }
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(use_intent.item).unwrap();
                            logger.append(format!("Used {} on {}.", item_name.name, mob_name.name));
                        }

                        is_item_used = true;
                    }
                }
            }

            //healing item logic
            let mut to_heal = Vec::<(Entity, (i32, i32))>::new(); 
            {
                let heals_component = heals_storage.get(use_intent.item);
                match heals_component {
                    None => {}
                    Some(h) => {
                        is_item_used = false;
                        for target in targets.iter() {
                            to_heal.push((*target, (h.duration, h.amount) )); 
                            is_item_used = true;
                        }
                    }
                }
            }
            for (e, c) in to_heal.iter() {
                healing_storage.insert(*e, Healing { duration: c.0, amount: c.1 })
                    .expect("Unable to insert Healing component.");
            }

            //confusion item logic
            let mut add_confusion = Vec::new();
            {
                let confuses = confusion.get(use_intent.item);
                match confuses {
                    None => {}
                    Some(confusion) => {
                        is_item_used = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(use_intent.item).unwrap();
                                logger.append(format!("Used {} on {}, they are confused!",
                                        item_name.name, mob_name.name));
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confusion.insert(mob.0, Confusion {turns: mob.1}).expect("Unable to insert status.");
            }

            //Magic Mapper Logic
            let mapper = magic_mapper.get(use_intent.item);
            match mapper {
                None => {}
                Some(_) => {
                    *runstate = RunState::MagicMapReveal{ row: 0 };
                    is_item_used = true;
                    logger.append("The Witness is within you.".to_string());
                }
            }
       
            //Delete used consumables
            if is_item_used {
                let consumable = consumables.get(use_intent.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(use_intent.item).expect("Delete failed.");
                    }
                }
            }
        }

        use_item_intent.clear();
        logger.log();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        ReadExpect<'a, Entity>,
                        WriteStorage<'a, DropItemIntent>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, UnequipIntent>,
                        ReadStorage<'a, Equipped>,
                        ReadStorage<'a, Name>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player_entity, mut drop_intent, mut positions, mut backpack,
             mut unequip_intents, equipped_storage, names) = data;

        let mut logger = gamelog::Logger::new();
        
        for (entity, drop_intent) in (&entities, &drop_intent).join() {
            let mut pos_to_drop: Position = Position {x: 0, y: 0};
            {
                let dropper_pos = positions.get(entity).unwrap();
                pos_to_drop.x = dropper_pos.x;
                pos_to_drop.y = dropper_pos.y;
            }
            positions.insert(drop_intent.item, Position {x: pos_to_drop.x, y: pos_to_drop.y})
                .expect("Unable to insert Position component.");
            backpack.remove(drop_intent.item);

            if let Some(_) = equipped_storage.get(drop_intent.item) {
               unequip_intents.insert(entity, UnequipIntent { item: drop_intent.item })
                   .expect("Unable to insert UnequipIntent component.");
            }

            if entity == *player_entity {
                logger.append(format!("{} dropped.", names.get(drop_intent.item).unwrap().name));
            }
        }

        drop_intent.clear();
        logger.log();
    }
}
