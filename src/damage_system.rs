use specs::prelude::*;
use std::cmp::max;
use bracket_lib::prelude::RandomNumberGenerator;
use super::{Stats, DamageQueue, DamageAtom, Player, Name, gui::gamelog,
            Resistances, RunState, Bleeding, particle_system::ParticleBuilder, Position};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, ParticleBuilder>,
                        WriteStorage<'a, Stats>,
                        WriteStorage<'a, DamageQueue>,
                        WriteStorage<'a, Bleeding>,
                        ReadStorage<'a, Resistances>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Position>,
                      );

    fn run (&mut self, data: Self::SystemData) {
        let (entities, mut particle_builder, mut stats, mut damage_queues, mut bleeding_storage,
             resistances, names, positions) = data;
        
        let mut to_bleed = Vec::<Entity>::new();
        let mut logger = gamelog::Logger::new();

        //Apply resistanes to dmg_queue and dmg_queue to stats.
        for (ent, name, pos, stats, d_q, res, bleeding) in
            (&entities, &names, &positions, &mut stats, &mut damage_queues,
             (&resistances).maybe(), (&bleeding_storage).maybe()).join() {
            
            //If this entity has resistances, apply them to damage_queue
            if let Some(resistance) = res { 
                for i in 0..d_q.queue.len() {
                    let d_atom = &d_q.queue[i];
                    match d_atom {
                        DamageAtom::Bludgeon(val) => {
                            d_q.queue[i] = DamageAtom::Bludgeon(val - resistance.bludgeon.value());
                        },
                        DamageAtom::Pierce(val) => {
                            d_q.queue[i] = DamageAtom::Pierce(val - resistance.pierce.value());
                        },
                        DamageAtom::Slash(val) => {
                            d_q.queue[i] = DamageAtom::Slash(val - resistance.pierce.value());
                        },
                        DamageAtom::Thermal(val) => {
                            d_q.queue[i] = DamageAtom::Thermal(val - resistance.thermal.value());
                        }
                        _ => {}
                    }
                }
            }
           
            //Check for bleeding
            let mut is_bleeding = false;
            if let Some(_) = bleeding { is_bleeding = true; }

            //Apply damage to stats
            let mut hp_dmg: i32 = 0;
            let mut fp_dmg: i32 = 0;
            let damage_iter = d_q.queue.iter();
            
            for dmg in damage_iter {
                match dmg {
                    DamageAtom::Bludgeon(n) |
                    DamageAtom::Pierce(n) |
                    DamageAtom::Slash(n) | 
                    DamageAtom::Thermal(n) => { hp_dmg += n; }
                    _ => { 
                        if (stats.fp - fp_dmg) > 0 {
                            fp_dmg += dmg.value();
                        } else {
                            hp_dmg += dmg.value();
                        }
                    }
                }
                
                if !is_bleeding && bleed_roll(dmg) { 
                    to_bleed.push(ent);
                    logger.append(format!("{} is bleeding.", &name.name));
                        
                }
            }
           
            if hp_dmg > 0 {
                stats.hp = max(0, stats.hp - hp_dmg);
                logger.append(format!("{} suffers {} damage.", &name.name, hp_dmg));

                //spawn particle
                particle_builder.request(pos.x, pos.y, bracket_lib::prelude::RGB::named(bracket_lib::prelude::ORANGE),
                    bracket_lib::prelude::RGB::named(bracket_lib::prelude::BLACK), bracket_lib::prelude::to_cp437('‼'), 200.0);
            }
            if fp_dmg > 0 {
                stats.fp = max(0, stats.fp - fp_dmg);
                logger.append(format!("{} suffers {} fatigue.", &name.name, fp_dmg));
            }

        }

        for e in to_bleed.iter() {
            bleeding_storage.insert(*e, Bleeding{}).expect("Unable to insert Bleeding component.");
        }

        damage_queues.clear();
        logger.log();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    
    { //scope in for borrow checker
        let stats = ecs.read_storage::<Stats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let names = ecs.read_storage::<Name>();
        let mut logger = gamelog::Logger::new();

        for (entity, stats) in (&entities, &stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let corpse_name = names.get(entity);
                        if let Some(corpse_name) = corpse_name {
                            logger.append(format!("{} has died.", &corpse_name.name));
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
        logger.log();
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete dead entity.");
    }
}

fn bleed_roll( dmg: &DamageAtom ) -> bool {
    let mut rng = RandomNumberGenerator::new();
    let mut result: bool = false;
    let bleed_range = rng.range(1,10);
    
    //dmg should be post-Resistance damage amount.
    match *dmg {
        DamageAtom::Slash(d) => if d > bleed_range { result = true; },
        DamageAtom::Pierce(d) => if d > bleed_range * 2 { result = true; },
        DamageAtom::Bludgeon(d) => if d > bleed_range * 4 { result = true; },
        _ => result = false
    }

    result
}
