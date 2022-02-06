use specs::prelude::*;
use super::{Position, JustMoved, EntryTrigger, Hidden, Map, Name, gamelog, DamageOnUse,
            DamageQueue, };

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = ( Entities<'a>,
                        ReadExpect<'a, Map>,
                        WriteStorage<'a, JustMoved>,
                        WriteStorage<'a, Hidden>,
                        WriteStorage<'a, DamageQueue>,
                        WriteStorage<'a, EntryTrigger>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, DamageOnUse>,
                        ReadStorage<'a, Name>,
                      );
    
    fn run(&mut self, data: Self::SystemData) {
       let (entities, map, mut moved_storage, mut hidden_storage, mut damage_queue,
            mut triggers, positions, damage_on_use, names) = data;

        let logger = gamelog::Logger::new();

       for (ent, pos, _ms) in (&entities, &positions, &mut moved_storage).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for entity in map.tile_content[idx].iter() {
                if ent != *entity { //check other ents in this tile for trigger, not self
                    let trigger = triggers.get(*entity);
                    match trigger {
                        None => {},
                        Some(trigger) => {
                            let name = names.get(*entity);
                            if let Some(name) = name {
                                logger.append(format!("Triggered a {}!", &name.name));
                            }

                            let damage = damage_on_use.get(*entity);
                            if let Some(d) = damage {
                                for atom in d.dmg_atoms.iter() {
                                    DamageQueue::queue_damage(&mut damage_queue, ent, *atom);
                                }
                            }

                            if !trigger.repeatable {
                                triggers.remove(*entity).expect("Unable to remove EntryTrigger component.");
                            }

                            hidden_storage.remove(*entity); // The trap is no longer hidden
                        }
                    }
                }
            }
       }

       moved_storage.clear();
       logger.log();
    }
}
