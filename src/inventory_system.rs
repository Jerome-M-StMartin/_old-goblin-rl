use specs::prelude::*;
use super::{WantsToPickUp, Name, InBackpack, Position, gamelog::GameLog,
            WantsToUseItem, Item, CombatStats, WantsToDropItem, Consumable,
            Heals, Damages, Damage, Ranged, Map, AoE};
use rltk::{console};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickUp>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for desire in wants_pickup.join() {
            positions.remove(desire.object);
            backpack.insert(desire.object, InBackpack { owner: desire.desired_by })
                .expect("Unable to insert item into backpack.");
            
            if desire.desired_by == *player_entity {
                gamelog.entries.push(format!("{} placed into inventory.",
                        names.get(desire.object).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Consumable>,
                        ReadStorage<'a, Heals>,
                        ReadStorage<'a, Damages>,
                        WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, Damage>,
                        ReadStorage<'a, AoE>
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, map, entities, mut wants_use, names,
             consumables, heals, damages, mut combat_stats, mut damage, aoe) = data;
        
        for (entity, useitem) in (&entities, &wants_use).join() {
            let mut is_item_used = true;
            let mut targets: Vec<Entity> = Vec::new();
           
            //targeting logic
            match useitem.target { 
                None => {targets.push(*player_entity);} //Assume Player is target.
                Some(target) => {
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => { //Single target
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => { //AoE
                            console::log("------------------It's an AoE!-------------");
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 &&
                                                   p.x < map.width - 1 &&
                                                   p.y > 0 &&
                                                   p.y < map.height - 1);
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                    console::log(targets.len().to_string());
                                }
                            }
                        }
                            
                    }
                }
            }

            //damaging item logic
            let damaging_item = damages.get(useitem.item);
            match damaging_item {
                None => {}
                Some(d) => {
                    let target_point = useitem.target.unwrap();
                    let idx = map.xy_idx(target_point.x, target_point.y);
                    is_item_used = false;
                    
                    for mob in map.tile_content[idx].iter() {
                        Damage::new_damage(&mut damage, *mob, d.dmg);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!("Used {} on {}: {} Damage!",
                                    item_name.name, mob_name.name, d.dmg));
                        }

                        is_item_used = true;
                    }
                }
            }

            //healing item logic
            let healing_item = heals.get(useitem.item);
            match healing_item {
                None => {}
                Some(healer) => {
                    is_item_used = false;
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("Used {}: Delta HP = {}",
                                        names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }

                            is_item_used = true;
                        }
                    }
                }
            }
        
            if is_item_used {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed.");
                    }
                }
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;
        
        for (entity, to_drop) in (&entities, &wants_drop).join() {
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

        wants_drop.clear();
    }
}
