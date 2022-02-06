use specs::prelude::*;
use super::{Hunger, RunState, HungerState, DamageQueue, gamelog, DamageAtom};

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
                        Entities<'a>,
                        ReadExpect<'a, Entity>, // The player
                        ReadExpect<'a, RunState>,
                        WriteStorage<'a, Hunger>,
                        WriteStorage<'a, DamageQueue>,
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, player, runstate, mut hunger_storage, mut damage_queues) = data;

        let logger = gamelog::Logger::new();

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
                                logger.append("Your belly grumbles.");
                            }
                        }
                        HungerState::Hungry => {
                            hunger.state = HungerState::Famished;
                            hunger.clock = 700;
                            if entity == *player {
                                logger.append("You are famished!");
                            }
                        }
                        HungerState::Famished => {
                            hunger.state = HungerState::Starving;
                            hunger.clock = 1000;
                            if entity == *player {
                                logger.append("You are starving.");
                            }
                        }
                        HungerState::Starving => {
                            if entity == *player {
                                logger.append("You are starving to death.");
                            }
                            DamageQueue::queue_damage(&mut damage_queues, entity, DamageAtom::Starvation);  
                        }
                    }
                }
            }
        }

        logger.log();
    }
}
