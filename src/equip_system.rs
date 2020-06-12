use specs::prelude::*;
use super::{EquipIntent, UnequipIntent, InBackpack, Equippable, Equipped, Weapon, BasicAttack,
            Resistances, Name, gamelog::GameLog};

pub struct EquipSystem {}

impl<'a> System<'a> for EquipSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, EquipIntent>,
                        WriteStorage<'a, UnequipIntent>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, BasicAttack>,
                        WriteStorage<'a, Resistances>,
                        ReadStorage<'a, Equippable>,
                        ReadStorage<'a, Weapon>,
                        ReadStorage<'a, Name>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, mut equipped, mut equip_intents, mut unequip_intents, mut in_backpack,
            mut basic_attacks, mut resistances, equippables, weapons, names) = data;

        //Equipping Logic:
        //This join will iterate over all living entities since Entitites is the only non-Maybe()
        //storage in the join(). This is OK here; equipping/unequipping is a rare event.
        for (owner, equip_intent, unequip_intent) in
            (&entities, (&mut equip_intents).maybe(), (&mut unequip_intents).maybe()).join() {
            
            //If owner entity has EquipIntent...
            if let Some(intent) = equip_intent {
                let ent_to_equip : Entity = intent.item;
                let target_slot = equippables.get(ent_to_equip).unwrap().slot; 
                let mut ent_to_unequip = None;
                

                //For each equipped equipment...
                for (entity, e) in (&entities, &equipped).join() {
                    //if there's an equip-slot collision...
                    if e.owner == owner && e.slot == target_slot {
                        ent_to_unequip = Some(entity);
                        break;
                    }
                }
                //if there was an equip-slot collision...
                if let Some(ent) = ent_to_unequip {
                   
                    //Unequip the old entity in this slot.
                    equipped.remove(ent);
                    in_backpack.insert(ent, InBackpack {owner})
                        .expect("Unable to insert InBackpack component.");               

                    //if unequipped entity has a Resistance component...
                    if let Some(resists_to_remove) = resistances.get(ent) {
                        *resistances.get_mut(owner).unwrap() =
                            *resistances.get(owner).unwrap() - *resists_to_remove;
                    }
                }

                in_backpack.remove(ent_to_equip);
                equipped.insert(ent_to_equip, Equipped {owner: owner, slot: target_slot})
                        .expect("Unable to insert Equipped component.");
                log.entries.push(format!("{} equipped {}.",
                        names.get(owner).unwrap().name, names.get(ent_to_equip).unwrap().name));


                //if equipped entity has a Weapon component...
                if let Some(w) = weapons.get(ent_to_equip) {// w == weapon
                    let (p, s, t) = (w.primary, w.secondary, w.tertiary);
                    match (p, s, t) {
                        (Some(primary), _, _) => {
                            BasicAttack::modify(&mut basic_attacks, owner, primary); },
                        (None, Some(secondary), _) => {
                            BasicAttack::modify(&mut basic_attacks, owner, secondary); },
                        (None, None, Some(tertiary)) => {
                            BasicAttack::modify(&mut basic_attacks, owner, tertiary); },
                        (_, _, _) => {}
                    }
                }

                //if equipped entity has a Resistance component...
                if let Some(resists_to_add) = resistances.get(ent_to_equip) {
                    *resistances.get_mut(owner).unwrap() =
                        *resistances.get(owner).unwrap() + *resists_to_add;
                }
            }

            //Unequip items for which current entity (owner) has UnequipIntent.
            if let Some(intent) = unequip_intent {
                let ent = intent.item;
                
                //Unequip it
                equipped.remove(ent);
                in_backpack.insert(ent, InBackpack {owner})
                    .expect("Unable to insert InBackpack component.");               

                //if unequipped entity has a Resistance component...
                if let Some(resists_to_remove) = resistances.get(ent) {
                    *resistances.get_mut(owner).unwrap() =
                        *resistances.get(owner).unwrap() - *resists_to_remove;
                }
                
                //if unequipped entity has a Weapon component...
                if let Some(_) = weapons.get(ent) {
                    BasicAttack::reset(&mut basic_attacks, owner);
                }
            }
        }
        
        unequip_intents.clear();
        equip_intents.clear();
    }
}
