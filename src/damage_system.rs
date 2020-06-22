use specs::prelude::*;
use std::cmp::max;
use super::{Stats, DamageQueue, DamageAtom, Player, Name, gamelog::GameLog,
            Resistances, RunState};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( ReadStorage<'a, Name>,
                        WriteStorage<'a, Stats>,
                        WriteStorage<'a, DamageQueue>,
                        ReadStorage<'a, Resistances>,
                        WriteExpect<'a, GameLog> );

    fn run (&mut self, data: Self::SystemData) {
        let (names, mut stats, mut damage_queues, resistances, mut log) = data;
        
        //Apply resistanes to dmg_queue and dmg_queue to stats.
        for (name, stats, d_q, res) in
            (&names, &mut stats, &mut damage_queues, (&resistances).maybe()).join() {
            
            //If this entity has resistances, apply them to damage_queue
            if let Some(resistance) = res { 
                for i in 0..d_q.queue.len() {
                    let d_atom = &d_q.queue[i];
                    match d_atom {
                        DamageAtom::Bludgeon(val) => {
                            d_q.queue[i] = DamageAtom::Bludgeon(val - resistance.bludgeon.value()); },
                        DamageAtom::Pierce(val) => {
                            d_q.queue[i] = DamageAtom::Pierce(val - resistance.pierce.value()); },
                        DamageAtom::Slash(val) => {
                            d_q.queue[i] = DamageAtom::Slash(val - resistance.pierce.value()); },
                        DamageAtom::Thermal(val) => {
                            d_q.queue[i] = DamageAtom::Thermal(val - resistance.thermal.value()); }
                    }
                }
            }
            
            //Apply DamageAtoms in damage_queue to stats
            let mut total_dmg: i32 = 0;
            let damage_iter = d_q.queue.iter();
            for dmg in damage_iter {
                match dmg {
                    DamageAtom::Pierce(dmg) => total_dmg += dmg,
                    DamageAtom::Slash(dmg) => total_dmg += dmg,
                    DamageAtom::Bludgeon(dmg) => total_dmg += dmg,
                    DamageAtom::Thermal(dmg) => total_dmg += dmg
                }
            }
            stats.hp = max(0, stats.hp - total_dmg);
            log.entries.push(format!("{} suffers {} damage.", &name.name, total_dmg));
        }

        damage_queues.clear()
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    
    { //scope in for borrow checker
        let stats = ecs.read_storage::<Stats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();

        for (entity, stats) in (&entities, &stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let corpse_name = names.get(entity);
                        if let Some(corpse_name) = corpse_name {
                            log.entries.push(format!("{} has died.", &corpse_name.name));
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
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete dead entity.");
    }
}
