//use std::cmp::max;
//use itertools::izip; //for zipping multiple iters into a single tuple output
use specs::prelude::*;
use super::{Rltk, UIColors, Inventory, MenuOption};
use rltk::RGB;

pub mod infocard;

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

enum UDLR { UP, DOWN, LEFT, RIGHT, }

struct MenuState {
    pub cursor_index: usize,
    pub history: Vec<UDLR>, //chain of selections, iterated each frame
    pub mr: MenuResult,
}

pub struct Menu {
    menu_chars: Vec<char>,
    state: Option<MenuState>,
}

impl Menu {
    fn new(origin: (i32, i32), width: u8, depth: u8) -> Menu {
       let mut m = Menu {
            menu_chars: Vec::new(),
            state: None,
       };

       let menu_size = width * depth;
       for _ in 0..menu_size {
            m.menu_chars.push('┌');
            m.menu_chars.push('•');
            m.menu_chars.push(' ');
       }

       return m;
    }   

    fn show_menu(ecs: &World, ctx: &mut Rltk, state: Option<MenuState>) {
        
    }

    fn show_infocard_only(ecs: &World, ctx: &mut Rltk, ent: Entity) {
        
    }
}

pub fn menu_builder(origin: (i32, i32)) -> Menu {
    let menu = Menu::new(origin, 3, 3);

    return menu;
}

fn draw_node_menu(ecs: &World, ctx: &mut Rltk, pos: (i32, i32), state: Option<MenuState>) /*-> MenuState*/ {

    //Separate into two functions:
    //fn1: build a new menu
    //fn1: draw an already-built menu

    let colors = ecs.fetch::<UIColors>();
    let player_ent = ecs.fetch::<Entity>();
    let inv = ecs.fetch::<Inventory>();
    let hands: &(Option<Entity>, Option<Entity>) = &inv.hands;
    let qkbar: &Vec<Option<Entity>> = &inv.quickbar;
    let bkpk: &Vec<Option<Entity>> = &inv.backpack;

    let mut to_print: Vec<char> = vec!['┌', '•', ' ', '┌', '•', ' ', '┌', '•', ' ',
                                   '├', '•', ' ', '├', '•', ' ', '├', '•', ' ',
                                   '└', '•', ' ', '└', '•', ' ', '└', '•', ' '];

    //HERE: Replace elements of to_print to match current history (state) of node menu,
    //such as the current cursor location and the path the user took to reach that location.
    if let Some(s) = state {
        
        //Iterate through history, compare dir to prev_dir to figure what <char> should
        //be drawn at curr_idx.
        let c_idx: usize = s.cursor_index;
        let mut curr_idx: usize = 0;
        let mut prev_dir: Option<UDLR> = None;
        for dir in s.history.iter().rev() {
            if curr_idx >= to_print.len() {
                panic!("curr_index ({}) out of bounds.", curr_idx);
            }
            match (dir, &prev_dir) {
                (UDLR::UP, Some(UDLR::UP)) => {
                    to_print[curr_idx] = '│';
                    curr_idx = udlr_to_idx(curr_idx, UDLR::UP);
                },
                (UDLR::UP, Some(UDLR::RIGHT)) => {
                    to_print[curr_idx] = '┘';
                    curr_idx = udlr_to_idx(curr_idx, UDLR::UP);
                    prev_dir = Some(UDLR::UP);
                },
                (UDLR::DOWN, None) => {
                    to_print[curr_idx] = '○';
                    curr_idx = udlr_to_idx(curr_idx, UDLR::DOWN);
                    prev_dir = Some(UDLR::DOWN);
                },
                (UDLR::DOWN, Some(UDLR::DOWN)) => {
                    to_print[curr_idx] = '│';
                    curr_idx = udlr_to_idx(curr_idx, UDLR::DOWN);
                },
                (UDLR::DOWN, Some(UDLR::RIGHT)) => {
                    to_print[curr_idx] = '┐';
                    curr_idx = udlr_to_idx(curr_idx, UDLR::DOWN);
                    prev_dir = Some(UDLR::DOWN);
                },
                (UDLR::RIGHT, None) => {
                    to_print[curr_idx] = '○';
                    to_print[curr_idx + 1] = '─';
                    to_print[curr_idx + 2] = '─';
                    prev_dir = Some(UDLR::RIGHT);
                },
                (UDLR::RIGHT, Some(UDLR::UP)) => {
                    to_print[curr_idx] = '┌';
                    to_print[curr_idx + 1] = '─';
                    to_print[curr_idx + 2] = '─';
                    curr_idx = udlr_to_idx(curr_idx, UDLR::RIGHT);
                    prev_dir = Some(UDLR::RIGHT);
                },
                (UDLR::RIGHT, Some(UDLR::DOWN)) => {
                    to_print[curr_idx] = '└';
                    to_print[curr_idx + 1] = '─';
                    to_print[curr_idx + 2] = '─';
                    curr_idx = udlr_to_idx(curr_idx, UDLR::RIGHT);
                    prev_dir = Some(UDLR::RIGHT);
                },
                (UDLR::RIGHT, Some(UDLR::RIGHT)) => {
                    to_print[curr_idx] = '─';
                    to_print[curr_idx + 1] = '─';
                    to_print[curr_idx + 2] = '─';
                    curr_idx = udlr_to_idx(curr_idx, UDLR::RIGHT);
                },
                (_, _) => { eprintln!("Illegal match input - cursor path will be wrong."); },
            }
        }

        to_print[c_idx + 1] = '►';
        to_print[c_idx + 2] = '♦';
    }

    //delineate slices for line-by-line printing
    let top_row: &[char] = &to_print[0..9];
    let mid_row: &[char] = &to_print[9..18];
    let bot_row: &[char] = &to_print[18..];

    let mut i = 0;
    loop {
        match i {
            0 => {
                let s: String = top_row.iter().collect();
                ctx.print_color(pos.0, pos.1, colors.main, RGB::named(rltk::BLACK), s);
            }
            1..=2 => {
                let s: String = mid_row.iter().collect();
                ctx.print_color(pos.0, pos.1, colors.main, RGB::named(rltk::BLACK), s);
            }
            _ => {
                let s: String = mid_row.iter().collect();
                ctx.print_color(pos.0, pos.1, colors.main, RGB::named(rltk::BLACK), s);
                break;
            }
        }
        i += 1;
    }
}

fn udlr_to_idx (from_idx: usize, dir: UDLR) -> usize {
    let mut new_idx: usize = from_idx;
    match dir {
        UDLR::UP => { new_idx = from_idx - 9; },
        UDLR::DOWN => { new_idx = from_idx + 9; },
        UDLR::LEFT => { new_idx = from_idx - 3; },
        UDLR::RIGHT => { new_idx = from_idx + 3; },
        _ => {},
    }

    return new_idx;
}

/*  ► = 16
 *  → = 26
 *  ↓ = 25
 *  ↑ = 24
 *  ♦ = 4
 *  • = 7
 *  ○ = 9
 *  ┌ = 218
 *  ├ = 195
 *  └ = 192
 *  ┼ = 197
 *  ┘ = 217
 *  ┐ = 191
 *  │ = 179
 *  ─ = 196
 *                  ┌• Potion
 *               ┌• ├• Shovel
 *       •┐   ┌• ├•→├-♦ Sword
 *        ├-@→├• ├• ├• Flint
 *       •┘   └•→├• ├○ EMPTY
 *               └○ ├○ EMPTY
 *      ┌•          └○ EMPTY
 *  ┌•  ├•
 *  ├•┌•├•
 *  └-┼->○
 *    ├•└•
 *    └•
 *
 *  ┌┬┬┬┬┐
 *  •••○••
 *    ┌┼┐
 *    •••
 *
 *  ┌○┐
 *  ├•├•
 *  └•├○┐
 *    └•├•
 *      └►♦
 *
 *
 *    ┌Hands
 *    │  ┌Quickbar
 *    │  │  ┌Backpack Layer 1
 *    ↓  ↓  ↓
 *  @→┌• ┌• ┌• Potion of Nullnoggin
 *  ↓ └•→├• ├• Dirty Shovel
 *  •┐   └•→├-♦ Bronze Dagger
 *  •┘      └• Woolen Socks
 *  ↑
 *  └On-Ground
 *
 */
