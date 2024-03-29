use specs::prelude::*;
use super::{gui::gamelog, ThrowIntent, Position, InBackpack, Equipped, Name, Weapon,
            DamageQueue, BasicAttack, Throwable, Map};

pub struct ThrowSystem {}

impl<'a> System<'a> for ThrowSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, ThrowIntent>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, BasicAttack>,
                        WriteStorage<'a, DamageQueue>,
                        WriteStorage<'a, Equipped>,
                        ReadStorage<'a, Throwable>,
                        ReadStorage<'a, Weapon>,
                        ReadStorage<'a, Name>,
                        ReadExpect<'a, Map>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut throw_intents, mut positions, mut in_backpack, mut basic_attacks,
             mut damage_queue, mut equipped_storage, throwables, weapons, names, map) = data;

        let mut logger = gamelog::Logger::new();

        for (ent, throw_intent) in (&entities, &mut throw_intents).join() {

            if let Some(target) = throw_intent.target {
                let throwable = throwables.get(throw_intent.item);
                let dmg = throwable.unwrap().dmg;
                let target_ent = None;

                let idx = map.xy_idx(target.x, target.y);
                if !map.tile_content[idx].is_empty() {
                    let target_ent = Some(map.tile_content[idx][0]);
                    DamageQueue::queue_damage(&mut damage_queue, target_ent.unwrap(), dmg);
                }
                
                if let Some(pos) = positions.get_mut(throw_intent.item) {
                    pos.x = target.x;
                    pos.y = target.y;
                } else {
                    positions.insert(throw_intent.item, Position { x: target.x, y: target.y })
                        .expect("Unable to insert Position component.");
                    
                    in_backpack.remove(throw_intent.item);
                    
                    //Can't use UnequipSystem because it puts unequipped things into backpacks.
                    if let Some(_) = equipped_storage.remove(throw_intent.item) {
                        if let Some(_) = weapons.get(throw_intent.item) {
                            BasicAttack::reset(&mut basic_attacks, ent);
                            for (equipped, weapon) in (&equipped_storage, &weapons).join() {
                                if equipped.owner == ent {
                                    BasicAttack::modify(&mut basic_attacks, ent, weapon.primary.unwrap());
                                }
                            }
                        }
                    }
                }

                if let Some(target_ent) = target_ent {
                    logger.append(format!("{} threw a {} at {}.",
                            names.get(ent).unwrap().name,
                            names.get(throw_intent.item).unwrap().name,
                            names.get(target_ent).unwrap().name));
                } else {
                    logger.append(format!("{} threw a {}.",
                            names.get(ent).unwrap().name,
                            names.get(throw_intent.item).unwrap().name));
                }
            }
        }

        throw_intents.clear();
        logger.log();
    }
}
