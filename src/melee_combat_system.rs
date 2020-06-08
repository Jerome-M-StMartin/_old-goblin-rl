use specs::prelude::*;
use super::{Stats, MeleeIntent, Name, DamageOnUse, DamageQueue, Equipped, gamelog::GameLog};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, MeleeIntent>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Stats>,
                        WriteStorage<'a, DamageQueue>,
                        ReadStorage<'a, DamageOnUse>,
                        ReadStorage<'a, Equipped>,
                        WriteExpect<'a, GameLog>
                      );
    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut melee_intent, names, stats,
            mut damage_queue, inflicts_damage, equipped, mut log) = data;

        //Weapon damage is changing to an Apply-On-Equip implementation;
        //when a weapon is equipped it applies its Primary bonus once to a Melee component.
        //Attack-Mode can be changed from a menu option.

        //Attach a  MeleeIntent component to all ents equipped to a creature with MeleeIntent.
        let mut relevant_equippables = Vec::<(Entity, Entity)>::new();
        for (entity, equipment, _inf_dmg, ()) in
            (&entities, &equipped, &inflicts_damage, !&melee_intent).join() {  
            
            let intent = &melee_intent.get(equipment.owner);
            if let Some(intent) = intent {
                relevant_equippables.push((entity, intent.target));
            }
        }
        for (ent, tgt) in relevant_equippables.iter() {
            melee_intent.insert(*ent, MeleeIntent {target: *tgt})
                .expect("MeleeIntent insertion failed.");
        }
        
        //Queue dmg from all living entities with MeleeIntent.
        for (entity, melee_intent, inflicts_damage, _name) in
            (&entities, &melee_intent, &inflicts_damage, &names).join() { 
            
            //if current entity is an equipment, return if its owner is dead
            let equipment = equipped.get(entity);
            if let Some(equipment) = equipment {
                if stats.get(equipment.owner).unwrap().hp <= 0 {return;}
            } else { //curr ent is creature; return if dead
                if stats.get(entity).unwrap().hp <= 0 {return;}
            }

            let target = melee_intent.target;
            let t_stats = stats.get(target).unwrap();

            if t_stats.hp > 0 {   
                let target_name = names.get(melee_intent.target).unwrap();
                
                for dmg_atom in inflicts_damage.dmg_atoms.iter() {
                    DamageQueue::queue_damage(&mut damage_queue, target, *dmg_atom);
                    /*log.entries.push(format!("{} deals {:?} Damage to {}",
                            &name.name, &dmg_atom, &target_name.name));*/
                } 
            }
        }
        
        melee_intent.clear();
    }
}
