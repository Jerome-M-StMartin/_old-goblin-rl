use specs::prelude::*;
use super::{PickUpIntent, Name, InBackpack, Position, gamelog::GameLog, UseItemIntent, Stats,
            DropItemIntent, Consumable, Heals, DamageOnUse, DamageQueue, Map, AoE, Confusion,
            Equippable, Equipped, Resistances, BasicAttack, Unequipped, Weapon, EquippedMap, EquipIntent};
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
        let (player_entity, mut gamelog, mut pickup_intent, mut positions, names, mut backpack) = data;

        for desire in pickup_intent.join() {
            positions.remove(desire.object);
            backpack.insert(desire.object, InBackpack { owner: desire.desired_by })
                .expect("Unable to insert item into backpack.");
            
            if desire.desired_by == *player_entity {
                gamelog.entries.push(format!("{} placed into inventory.",
                        names.get(desire.object).unwrap().name));
            }
        }

        pickup_intent.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, UseItemIntent>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Consumable>,
                        ReadStorage<'a, Heals>,
                        ReadStorage<'a, DamageOnUse>,
                        WriteStorage<'a, Stats>,
                        WriteStorage<'a, DamageQueue>,
                        ReadStorage<'a, AoE>,
                        WriteStorage<'a, Confusion>,
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, EquipIntent>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, map, entities, mut use_item_intent, names,
             consumables, heals, inflicts_damage, mut stats, mut damage_queue,
             aoe, mut confusion, equippable, mut equip_intent) = data;
        
        for (entity, use_intent, equipment) in (&entities, &use_item_intent, (&equippable).maybe()).join() {
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
                            }
                        }
                            
                    }
                }
            }
            
            //If item with use_intent is equippable, insert equip intent.
            //Further logic for this item is to be handled by EquipSystem.
            if let Some(_e) = equipment {
                equip_intent.insert(entity, EquipIntent {wearer: targets[0]})
                    .expect("Failed to insert EquipIntent component.");
                continue;
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
            let healing_item = heals.get(use_intent.item);
            match healing_item {
                None => {}
                Some(healer) => {
                    is_item_used = false;
                    for target in targets.iter() {
                        let stats = stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("Used {}: Delta HP = {}",
                                        names.get(use_intent.item).unwrap().name, healer.heal_amount));
                            }

                            is_item_used = true;
                        }
                    }
                }
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
        
        for (entity, to_drop) in (&entities, &drop_intent).join() {
            let mut dropper_pos: Position = Position {x: 0, y: 0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position {x: dropper_pos.x, y: dropper_pos.y})
                .expect("Unable to insert dropped item position.");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("{} dropped.", names.get(to_drop.item).unwrap().name));
            }
        }

        drop_intent.clear();
    }
}

pub struct EquipSystem {}

impl<'a> System<'a> for EquipSystem {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, Unequipped>,
                        WriteStorage<'a, EquipIntent>,
                        WriteStorage<'a, InBackpack>,
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, EquippedMap>,
                        ReadStorage<'a, Weapon>,
                        WriteStorage<'a, BasicAttack>
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut equipped, mut unequipped, mut equip_intents, mut in_backpack,
             equippables, mut equipped_maps, weapons,  mut basic_attacks) = data;
        
        //Equipping Logic
        for (entity, equippable, equip_intent, weapon) in
            (&entities, &equippables, &mut equip_intents, (&weapons).maybe()).join() {
            
            let target_owner = equip_intent.wearer;
            let target_slot = equippable.slot;
            //let equips_map = equipped_maps.get_mut(target_owner).unwrap();

            //Try equipping
            if let Some(formerly_equipped) =
                //equips_map.equip(equipped_maps, target_owner, target_slot, entity) {
                EquippedMap::equip(&mut equipped_maps, target_owner, target_slot, entity) {
                
                equipped.remove(formerly_equipped);
                unequipped.insert(formerly_equipped, Unequipped {}).
                    expect("Failed to insert Unequipped component.");
                in_backpack.insert(formerly_equipped, InBackpack {owner: target_owner})
                    .expect("Failed to insert InBackpack component.");
            } else { //entity successfully inserted into hashmap without collision
                in_backpack.remove(entity);
                equipped.insert(entity, Equipped {owner: target_owner, slot: target_slot})
                    .expect("Failed to insert Equipped component.");
            }
            
            //"OnEquip" Weapon logic
            if let Some(weapon) = weapon {
                if let Some(primary) = weapon.primary {
                    BasicAttack::modify(&mut basic_attacks, target_owner, primary);
                    //let &mut base_atk = basic_attacks.get(target_owner).unwrap();
                    //base_atk.modify(primary);
                }
            }
        }

        equip_intents.clear();                

        //Unequipping Logic
        for (_unequipped, in_backpack, weapon) in
            (&mut unequipped, &in_backpack, (&weapons).maybe()).join() {
            
            //Weapon unequip logic
            if let Some(_w) = weapon {
                BasicAttack::reset(&mut basic_attacks, in_backpack.owner);
                //let  &mut base_atk = basic_attacks.get(in_backpack.owner).unwrap();
                //base_atk.reset();
            }
        }

        unequipped.clear();

    }
}
            /*//equip if equippable & unequip whatever else was equipped in the slot, if any
            let equip_item = equippable.get(use_intent.item);
            match equip_item {
                None => {}
                Some(item_to_equip) => {
                    let target_slot = item_to_equip.slot;
                    let target = targets[0];

                    //Remove any items the target has in the item's slot
                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequip {}.", name.name));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack.insert(*item, InBackpack {owner: target})
                            .expect("Unable to insert backpack entry.");
                    }

                    //wield item
                    equipped.insert(use_intent.item, Equipped {owner: target, slot: target_slot})
                        .expect("Unable to insert equipped component.");
                    backpack.remove(use_intent.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!("You equip {}.", names.get(use_intent.item)
                                .unwrap().name));
                    }
                }
            }*/

