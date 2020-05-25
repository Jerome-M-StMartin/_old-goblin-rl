use specs::prelude::*;
use super::{CombatStats, Damage, Player, Name, gamelog::GameLog};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, Damage> );

    fn run (&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }
        
        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    
    { //scope in for borrow checker (seems like bad form ???)
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();

        for (entity, stats) in (&entities, &combat_stats).join() {
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
                    Some(_) => log.entries.push("------MORTIS------".to_string())
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete dead entity.");
    }
}
