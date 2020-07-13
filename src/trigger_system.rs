use specs::prelude::*;
use super::{Position, JustMoved, EntryTrigger, Hidden, Map, Name, gamelog::GameLog};

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>,
                        WriteStorage<'a, JustMoved>,
                        WriteStorage<'a, Hidden>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, EntryTrigger>,
                        ReadStorage<'a, Name>,
                      );
    
    fn run(&mut self, data: Self::SystemData) {
       let (entities, mut log, map, mut moved_storage, mut hidden_storage, positions, triggers, names) = data;

       for (ent, pos, _ms) in (&entities, &positions, &mut moved_storage).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for entity in map.tile_content[idx].iter() {
                if ent != *entity { //don't check self for trap-ness
                    let maybe_trigger = triggers.get(*entity);
                    match maybe_trigger {
                        None => {},
                        Some(_) => {
                            let name = names.get(*entity);
                            if let Some(name) = name {
                                log.entries.push(format!("Triggered a {}!", &name.name));
                            }

                            hidden_storage.remove(*entity); // The trap is no longer hidden
                        }
                    }
                }
            }
       }

       moved_storage.clear();
    }
}
