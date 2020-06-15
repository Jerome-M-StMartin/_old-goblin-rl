use specs::prelude::*;
use super::{Stats, MeleeIntent, DamageQueue, BasicAttack};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, MeleeIntent>, 
                        WriteStorage<'a, DamageQueue>,
                        ReadStorage<'a, BasicAttack>,
                        ReadStorage<'a, Stats>,
                      );
    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut melee_intents, mut damage_queues, basic_attacks, stats) = data;

        //Queue dmg from all living entities with MeleeIntent.
        for (entity, melee_intent, basic_attack) in
            (&entities, &melee_intents, &basic_attacks).join() { 
            
            //If entity with intent is dead, well... they can't melee.
            if stats.get(entity).unwrap().hp <= 0 {return;}

            let target = melee_intent.target;
            let t_stats = stats.get(target).unwrap();

            if t_stats.hp > 0 {   
                DamageQueue::queue_damage(&mut damage_queues, target, basic_attack.current);
            }
        }
        
        melee_intents.clear();
    }
}
