use specs::prelude::*;
use super::{PickUpIntent, Name, InBackpack, Position, gamelog::GameLog, UseItemIntent,
            DropItemIntent, Consumable, Healing, Heals, DamageOnUse, DamageQueue, Map, AoE, Confusion,
            Equippable, Equipped, EquipIntent, UnequipIntent, particle_system::ParticleBuilder};
//use rltk::{console};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, PickUpIntent>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut pickup_intents, mut positions,
             names, mut in_backpack) = data;

        for intent in pickup_intents.join() {
            //If item is not already picked-up...(Is Some if item has position, else None).
            if let Some(_) = positions.remove(intent.item) {
                in_backpack.insert(intent.item, InBackpack { owner: intent.desired_by })
                    .expect("Unable to insert item into backpack.");
            
                if intent.desired_by == *player_entity {
                    gamelog.entries.push(format!("{} placed into inventory.", 
                            names.get(intent.item).unwrap().name));
                }
            }
        }

        pickup_intents.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, ParticleBuilder>,
                        WriteExpect<'a, GameLog>,
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
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, EquipIntent>,
                        ReadStorage<'a, Equipped>,
                        WriteStorage<'a, UnequipIntent>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut particle_builder, mut gamelog, map, entities, mut use_item_intent, names,
             consumables, mut healing_storage, heals_storage, inflicts_damage, mut damage_queue,
             aoe, mut confusion, equippable, mut equip_intent, equipped, mut unequip_intent) = data;

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
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
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
                                    rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('░'), 200.0);
                            }
                        }
                            
                    }
                }
            }

            //If item is equippable, insert EquipIntent or UnequipIntent.
            //Further logic for this item is to be handled by EquipSystem.
            if let Some(_e) = equippable.get(use_intent.item) {//if equippable
                if let Some(_e) = equipped.get(use_intent.item) {//if already equipped
                    unequip_intent.insert(entity, UnequipIntent {item: use_intent.item})
                        .expect("Failed to insert UnequipIntent component.");
                } else {//if not yet equipped
                    equip_intent.insert(entity, EquipIntent {item: use_intent.item})
                        .expect("Failed to insert EquipIntent component.");
                }
            }

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
                            gamelog.entries.push(format!("Used {} on {}.",
                                    item_name.name, mob_name.name));
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
                                gamelog.entries.push(format!("Used {} on {}, they are confused!",
                                        item_name.name, mob_name.name));
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confusion.insert(mob.0, Confusion {turns: mob.1}).expect("Unable to insert status.");
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
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, DropItemIntent>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut drop_intent, names, mut positions, mut backpack) = data;
        
        for (entity, drop_intent) in (&entities, &drop_intent).join() {
            let mut dropper_pos: Position = Position {x: 0, y: 0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(drop_intent.item, Position {x: dropper_pos.x, y: dropper_pos.y})
                .expect("Unable to insert dropped item position.");
            backpack.remove(drop_intent.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("{} dropped.", names.get(drop_intent.item).unwrap().name));
            }
        }

        drop_intent.clear();
    }
}
