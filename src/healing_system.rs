use specs::prelude::*;
use std::cmp::{min};
use super::{Name, gui::gamelog, Stats, Healing, Bleeding};
//use bracket_lib::prelude::{console};

pub struct HealingSystem {}

impl<'a> System<'a> for HealingSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, Healing>,
                        WriteStorage<'a, Stats>,
                        WriteStorage<'a, Bleeding>,
                        ReadStorage<'a, Name>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut healing_storage, mut stats, mut bleeding_storage, names) = data;

        let mut remove_bleed = Vec::<Entity>::new();
        let mut logger = gamelog::Logger::new();

        for (ent, mut healing, mut stats, name, bleeding) in
            (&entities, &mut healing_storage, &mut stats, &names, (&bleeding_storage).maybe()).join() {
            
            if healing.duration > 0 {
                healing.duration -= 1;
                
                if let Some(_) = bleeding {
                    remove_bleed.push(ent);
                    logger.append(format!("{} stopped bleeding.", &name.name));
                } else {
                    stats.hp = min(stats.max_hp, stats.hp + healing.amount);
                    logger.append(format!("{} gained {} HP.", &name.name, healing.amount));
                    if healing.duration <= 0 { remove_bleed.push(ent); }
                }
            }

        }

        for e in remove_bleed.iter() {
            bleeding_storage.remove(*e);
        }

        logger.log();
    }
}
