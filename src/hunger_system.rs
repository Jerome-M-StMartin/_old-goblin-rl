use specs::prelude::*;
use super::{Hunger, RunState, HungerState, DamageQueue, gamelog::GameLog, DamageAtom};

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
                        Entities<'a>,
                        ReadExpect<'a, Entity>, // The player
                        ReadExpect<'a, RunState>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, Hunger>,
                        WriteStorage<'a, DamageQueue>,
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, player, runstate, mut log, mut hunger_storage, mut damage_queues) = data;

        for (entity, mut hunger) in (&entities, &mut hunger_storage).join() {
            let mut proceed = false;

            match *runstate {
                RunState::PlayerTurn => if entity == *player { proceed = true; },
                RunState::GameworldTurn => if entity != *player { proceed = true; },
                _ => proceed = false
            }

            if proceed {
                hunger.clock -= 1;
                if hunger.clock < 1 {
                    match hunger.state {
                        HungerState::Stuffed => {
                            hunger.state = HungerState::Satiated;
                            hunger.clock = 400;
                        }
                        HungerState::Satiated => {
                            hunger.state = HungerState::Hungry;
                            hunger.clock = 500;
                            if entity == *player {
                                log.entries.push("Your belly grumbles.".to_string());
                            }
                        }
                        HungerState::Hungry => {
                            hunger.state = HungerState::Famished;
                            hunger.clock = 700;
                            if entity == *player {
                                log.entries.push("You are famished!".to_string());
                            }
                        }
                        HungerState::Famished => {
                            hunger.state = HungerState::Starving;
                            hunger.clock = 1000;
                            if entity == *player {
                                log.entries.push("You are starving.".to_string());
                            }
                        }
                        HungerState::Starving => {
                            if entity == *player {
                                log.entries.push("You are starving to death.".to_string());
                            }
                            DamageQueue::queue_damage(&mut damage_queues, entity, DamageAtom::Starvation);  
                        }
                    }
                }
            }
        }
    }
}
