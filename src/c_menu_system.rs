use specs::prelude::*;
use super::{Menuable, MenuOption, Item, Position, Equippable, Hostile};
pub struct ContextMenuSystem {}

impl<'a> System<'a> for ContextMenuSystem {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, Menuable>,
                        ReadStorage<'a, Item>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Equippable>,
                        ReadStorage<'a, Hostile>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut menu_options,
            items, positions, equippable, monster) = data;

        //Populate Menuable components.
        for (ent, _pos, menu) in (&entities, &positions, &mut menu_options).join() {
            if menu.dirty {
                if let Some(_) = items.get(ent) {
                    menu.options.push( (MenuOption::Use, "Use".to_string()) );
                    
                    if let Some(_) = positions.get(ent) { menu.options
                        .push( (MenuOption::PickUp, "Pick Up".to_string()) ); }
                    
                    if let Some(_) = equippable.get(ent) { menu.options
                        .push( (MenuOption::Equip, "Equip".to_string()) ); }
                }

                if let Some(_) = monster.get(ent) { menu.options
                    .push( (MenuOption::Attack, "Attack".to_string()) ); }

            }
            menu.dirty = false;
        }
    }
}
