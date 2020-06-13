use rltk::{Rltk, RGB, VirtualKeyCode};
use specs::prelude::*;
use super::{Name, gamelog::GameLog, State, RunState, ContextMenuOptions, Item, Position, InBackpack, Weapon};
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MenuOption {
    PickUp,
    DropIt,
    Use,
    Equip,
    Attack,
    //Examine
}

#[derive(PartialEq, Copy, Clone)]
pub enum MenuResult {
    NoSelection { option: MenuOption },
    Selection { option: MenuOption }
}

pub struct ContextualMenuSystem {}

impl<'a> System<'a> for ContextualMenuSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, ContextMenuOptions>,
                        ReadStorage<'a, Item>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, InBackpack>,
                        ReadStorage<'a, Weapon>
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, names, mut option_hashes,
            items, positions, in_backpack, weapons) = data;

        for (ent, name, options) in (&entities, &names, &mut option_hashes).join() {
            
            //populate options component if is_empty()
            if options.options.is_empty() {
                //if ent is an item, it can be used.
                if let Some(_) = items.get(ent) { options.options.push(MenuOption::Use); }
                if let Some(_) = positions.get(ent) { options.options.push(MenuOption::PickUp); }
                if let Some(_) = in_backpack.get(ent) { options.options.push(MenuOption::DropIt); }
                if let Some(_) = weapons.get(ent) { options.options.push(MenuOption::Attack); }
            }
        
        }
    }
}

/*
//maybe this needs to be moved back into gui...
//but in that case it should basically only handle the drawing
//and needs to have the target Entity passed in.
pub fn show_context_menu(gs: &mut State, ctx: &mut Rltk) -> ContexMenuResult {
 


    let runstate = gs.ecs.fetch::<RunState>();
    let options: <Vec<CtxMenuOption>> = Vec::new();

    //determine what CtxMenuOptions apply to this Entity

    if let Runstate::ContextMenu{ curr<CtxMenuResult>: option } = *runstate {
        if option == CtxMenuOption::PickUp {
            //draw highlighted PickUp text
        } else {
            //draw UNhighlighted PickUp text
        }
    } //etc...
}*/


