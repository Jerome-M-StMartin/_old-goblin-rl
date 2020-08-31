use specs::prelude::*;
use super::{Rltk, RGB};

mod infocard;
use infocard;


/*
 * Concept:
 * Module to handle ALL in-game menus.
 * For on-map Cursor access to infocards, a call to 'show_infocard' will display the infocard
 * on-screen without the node-menu visible.
 * For in-inventory access to infocards, a call to 'show_inventory' will display the node-menu and
 * also display the infocard for the currently selected entity in inventory. By default, when the
 * node-menu is first shown and no selection has been made, the infocard shows the Player data.
 * Thus, the the 'Character Sheet' is found in the same place as any other info the player needs.
 */


pub enum MenuResult {
    Continue, //selection not yet made, keep menu open in curr state.
    Cancel, //close menu (available from any state)
    Selected { selection: (MenuOption, Entity) }, //execute selected action & KEEP MENU OPEN
}

pub struct MenuState {
    pub history: Vec<u8>, //chain of selections, iterated each frame
    pub mr: MenuResult,
}

struct Menu {}

impl Menu {
    fn show_menu(ecs: &World, ctx: &mut Rltk, state: Option<MenuState>) -> MenuState {
        if let Some(state) = state {
            draw_nodes(state.history); //draw nodes w/ history highlights

        } else { //no prev. state
            let player_ent = ecs.fetch::<Entity>();
            self.show_infocard(ecs, ctx, *player_ent);
            draw_nodes(&ecs, &mut ctx, None);
        }
    }

    fn show_infocard(ecs: &World, ctx: &mut Rltk, ent: Entity) {
        infocard::show_infocard(ecs, ctx, ent);
    }
}

fn draw_nodes(ecs: &World, ctx: &mut Rltk, pos: (i32, i32), history: Option<Vec<u8>>) {

    let colors = ecs.fetch::<Colors>();
    let player_ent = ecs.fetch::<Entity>();
    let inv = ecs.fetch::<Inventory>();
    let bkpk: Vec<Entity> = inv.backpack;
    let eqpmt: Vec<Entity> = inv.equipment;

    //would be COOL AF if the menu node visually matched the glyph from the renderable component of
    //the item in that menu slot.
    let renderables = ecs.read_storage::<Renderable>();

    //bkpk and eqpmt represent COLUMNS of the node-menu
    //I need a way to break this down into ROWS to minimize calls to ctx.draw().

    if let Some(h) = history {
        
    } else {
        ctx.print_color(
    }

    //Display node-menu in current state (as indicated by 'history')
    //draw first upisde-down 'L' + NODE: (218) + (9 {or 8 for "curr_selection"})
    //draw each sideways 'T' + NODE: (195) + (9)
    //draw final 'L' + NODE: (192) + (9)
    ctx.print_color(pos.0, pos.1, colors.main, RGB::named(rltk::BLACK), );
    
}
