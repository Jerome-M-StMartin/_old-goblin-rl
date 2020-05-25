use specs::prelude::*;
use super::{CombatStats, WantsToMelee, Name, Damage, gamelog::GameLog};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, WantsToMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, Damage>,
                        WriteExpect<'a, GameLog>
                      );
    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_melee, names, combat_stats, mut damage_queue, mut log) = data;

        for (_entity, wants_melee, name, stats) in
        (&entities, &wants_melee, &names, &combat_stats,).join() {
            
            if stats.hp > 0 {
                 let target_stats = combat_stats.get(wants_melee.target).unwrap();
                    
                if target_stats.hp > 0 {   
                    let target_name = names.get(wants_melee.target).unwrap();
                    let damage = i32::max(0, stats.power - target_stats.defense);
                    
                    if damage == 0 {
                        log.entries.push(format!("{} is unable to hurt {}", &name.name, target_name.name));
                    } else {
                        Damage::new_damage(&mut damage_queue, wants_melee.target, damage);
                        log.entries.push(format!("{} deals {} DMG to {}!", &name.name, &damage, target_name.name));
                    }
                }
            }
        }
            
        wants_melee.clear();
    }
}
