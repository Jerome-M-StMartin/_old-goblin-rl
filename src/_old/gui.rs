use bracket_lib::prelude::{ RGB, Rltk, Point, VirtualKeyCode, INPUT };
use specs::prelude::*;
use std::cmp::{max, min};
use super::{ Map, Stats, Player, Name, Position, gamelog::GameLog, State, InBackpack,
             Viewshed, RunState, Equipped, Menuable, MenuOption, Cursor, Hidden, camera,
             menu::infocard, };

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection {selected: MainMenuSelection},
    Selected {selected: MainMenuSelection}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MenuResult { Continue, Cancel, Selected }


pub fn show_infocard(ecs: &World, ctx: &mut Rltk, ent: Entity) {
    infocard::show_infocard(ecs, ctx, ent);
}


pub fn open_context_menu(ecs: &World, ctx: &mut Rltk, selection: i8, focus: i8) ->
                                (MenuResult, Option<(MenuOption, Entity)>, i8, i8) {
   
    let mut result = (MenuResult::Continue, None, selection, focus);
    
    let cursor = ecs.fetch::<Cursor>();
    let map = ecs.fetch::<Map>();
    let contents = &map.tile_content[map.xy_idx(cursor.x, cursor.y)];
    
    if contents.len() > focus as usize { //Is there context for the menu?
        let ent = contents[focus as usize]; //Menu Context
        let menuable = ecs.read_storage::<Menuable>();
        
        if let Some(menuable) = &menuable.get(ent) { //Is Context Menuable?
            if !&menuable.options.is_empty() { //Is Menuable component populated?
            
                let num_options = menuable.options.len() as i8;
                let height = num_options + 1; 

                ctx.draw_box(1, 1, 14, height, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK));
                ctx.print_color(2, 1, RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), "Context Menu");

                //Draw the menu options.
                let mut y = 1;
                for (_, s) in &menuable.options {
                    ctx.print_color(3, y + 1, RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), s);
                    y += 1;
                    
                }
                
                //Query player for choice.
                match ctx.key {
                    None => {}
                    Some(key) => match key {
                        
                        VirtualKeyCode::Escape |
                        VirtualKeyCode::C => { result = (MenuResult::Cancel, None, 0, 0) },

                        VirtualKeyCode::W |
                        VirtualKeyCode::Up |
                        VirtualKeyCode::Numpad8 |
                        VirtualKeyCode::K => { result = (MenuResult::Continue, None,
                                                         max(0, selection - 1), focus) }

                        VirtualKeyCode::S |
                        VirtualKeyCode::Down |
                        VirtualKeyCode::Numpad2 |
                        VirtualKeyCode::J => { result = (MenuResult::Continue, None,
                                                         min(num_options - 1, selection + 1), focus ) }                                   
                        VirtualKeyCode::Tab => { result = (MenuResult::Continue, None, 0, focus + 1) }

                        VirtualKeyCode::Return |
                        VirtualKeyCode::NumpadEnter |
                        VirtualKeyCode::E => {
                            result = ( MenuResult::Selected,
                                       Some( (menuable.options[selection as usize].0, ent) ),
                                       0,
                                       0 );
                        }
                        _ => {}
                    }
                }

                //Disallow menu use if target is too far from player for interaction.
                if !is_adjacent_to_player(ecs, cursor.x, cursor.y) {
                    ctx.print_color(3, height + 1, RGB::named(bracket_lib::prelude::RED),
                                                   RGB::named(bracket_lib::prelude::BLACK), "TOO FAR AWAY");
                    if result.0 != MenuResult::Cancel { result = (MenuResult::Continue, None, selection, focus); }
                } else {
                    //Draw the curr-selction arrow.
                    ctx.print_color(1, selection + 2, RGB::named(bracket_lib::prelude::MAGENTA), RGB::named(bracket_lib::prelude::BLACK), "->");
                }
            } else { result = (MenuResult::Cancel, None, 0, 0); }
        } else { result = (MenuResult::Cancel, None, 0, 0); }

    } else if focus > 0 { //Loop focus back to first entity on the tile.
        result = open_context_menu(ecs, ctx, 0, 0);
    } else { result = (MenuResult::Cancel, None, 0, 0); }

    return result;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerMenuState {
    pub selection: i8,
    pub focus: i8,
    pub mr: MenuResult,
    pub substate: Option<SubState>,
    pub result: Option<(MenuOption, Entity)>
}

impl Default for PlayerMenuState {
    fn default() -> PlayerMenuState {
            PlayerMenuState {
            selection: 0,
            focus: 0,
            mr: MenuResult::Continue,
            substate: None,
            result: None
        }
    }
}

pub fn open_player_menu(ecs: &World, ctx: &mut Rltk, mut state: PlayerMenuState) ->
                                                                    PlayerMenuState {
    let entities = ecs.entities();
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let equipped = ecs.read_storage::<Equipped>();

    let items_in_bkpk = (&backpack).join().filter(|item| item.owner == *player_entity).count() as i32;
    let items_equipped = (&equipped).join().filter(|item| item.owner == *player_entity).count() as i32;


    let x = 5;
    let scale = ctx.get_scale(); //(f32, i32, i32) == (scale, center_x, center_y)
    let menu_width = (scale.1) - (x * 2);
    let max_menu_height = (scale.2 * 2) - (x * 2);
    
    let mut box_top = 0;
    let mut box_bottom = 0;
    let mut y: i32;
    let mut selection_tracker = 0;
    let mut curr_ent = *player_entity; //Only set to p_ent to avoid "possibly uninitialized" err.
    let mut num_options = 0;

    if state.focus > 1 { state.focus = 0; }
    if state.focus < 0 { state.focus = 1; }

    //Based on curr focus, draw menu box & populate with relevant selections...
    match state.focus {
        0 => {
            y = (max_menu_height / 2) - (items_in_bkpk / 2);
            box_top = y-2;
            box_bottom = items_in_bkpk + 3;
            
            ctx.draw_box(x, box_top, menu_width, box_bottom,
                RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK));           
            ctx.print_color(menu_width / 2, box_top,
                RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), "Backpack");
           
            let mut ent_chosen = false;
            for (ent, name, _in_pack) in (&entities, &names, &backpack).join()
                                     .filter(|tuple| tuple.2.owner == *player_entity) {
                
                ctx.print(x + 2, y, &name.name.to_string());
                
                if !ent_chosen && selection_tracker == state.selection {
                    curr_ent = ent;
                    ent_chosen = true;
                } else {
                    selection_tracker += 1;
                }
                
                num_options += 1;
                y += 1;

            }
        }
        1 => {
            y = (max_menu_height / 2) - (items_equipped / 2);
            box_top = y-2;
            box_bottom = items_equipped + 3;

            ctx.draw_box(x, box_top, menu_width, box_bottom,
                RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK));
            ctx.print_color(menu_width / 2, box_top,
                RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), "Equipment");

            let mut ent_chosen = false;
            for (ent, name, _equipment) in (&entities, &names, &equipped).join()
                                  .filter(|tuple| tuple.2.owner == *player_entity) {
                
                ctx.print(x + 2, y, &name.name.to_string());
                
                if !ent_chosen && selection_tracker == state.selection {
                    curr_ent = ent;
                    ent_chosen = true;
                } else {
                    selection_tracker += 1;
                }
                
                num_options += 1;
                y += 1;
            }
        }
        _ => {}
    }
 
    ctx.print_color(x+2, box_top + box_bottom,
        RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), "TAB: Next Page");
    ctx.print_color(x+18, box_top + box_bottom,
        RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), "ESC: Close");
    
    if num_options > 0 {
        ctx.print_color(x, box_top + state.selection as i32 + 2,
            RGB::named(bracket_lib::prelude::MAGENTA), RGB::named(bracket_lib::prelude::BLACK), "->");
    } else {
        ctx.print_color(x + 2, box_top + 2,
            RGB::named(bracket_lib::prelude::RED), RGB::named(bracket_lib::prelude::BLACK), "Empty");
    }

    //Query player for choice.
    if let Some(substate) = state.substate {
        
        let new_substate = open_submenu(&ecs, ctx, substate);
        
        match new_substate.sub_mr {
            MenuResult::Continue => { state.substate = Some(new_substate); }
            MenuResult::Cancel => { state.substate = None; }
            MenuResult::Selected => {
                if let Some(result) = new_substate.result { 
                    state.result = Some(result);
                    state.mr = MenuResult::Selected;
                }
            }
        }
    } else {
        match ctx.key {
            None => {}
            Some(key) => match key {
              
                VirtualKeyCode::Escape |
                VirtualKeyCode::C => { state.mr = MenuResult::Cancel; }

                VirtualKeyCode::W |
                //VirtualKeyCode::Up |
                //VirtualKeyCode::Numpad8 |
                VirtualKeyCode::K => { state.selection = max(0, state.selection - 1); }

                VirtualKeyCode::S |
                //VirtualKeyCode::Down |
                //VirtualKeyCode::Numpad2 |
                VirtualKeyCode::J => { state.selection = min(num_options - 1, state.selection + 1); }

                VirtualKeyCode::Tab => { state.focus += 1; }

                VirtualKeyCode::Return |
                //VirtualKeyCode::NumpadEnter |
                VirtualKeyCode::E => {
                    let init_substate = SubState {
                                        e: curr_ent,
                                        pos: (x+2, box_top + selection_tracker as i32 + 3),
                                        sub_selection: 0,
                                        sub_mr: MenuResult::Continue,
                                        result: None };
                    state.substate = Some(init_substate);
                }
                _ => {}
            }
        }
    }

    return state;
}

//Lives inside of PlayerMenuState struct only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubState {
    pub e: Entity,
    pub pos: (i32, i32),
    pub sub_selection: i8,
    pub sub_mr: MenuResult,
    pub result: Option<(MenuOption, Entity)>,
}

fn open_submenu(ecs: &World, ctx: &mut Rltk, mut state: SubState) -> SubState {

    let menuable_storage = ecs.read_storage::<Menuable>();
    
    if let Some(menuable) = &menuable_storage.get(state.e) { //Is Context Menuable?
        if !&menuable.options.is_empty() { //Is Menuable component populated?
        
            let num_options = menuable.options.len() as i8;
            let height = num_options + 1;
            let x = state.pos.0;
            let mut y = state.pos.1;

            ctx.draw_box(x, y, 13, height, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK));

            //Draw the menu options.
            for (_, s) in &menuable.options {
                ctx.print_color(x + 2, y + 1, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK), s);
                y += 1;
                
            }
            
            //Query player for choice.
            match ctx.key {
                None => {}
                Some(key) => match key {
                    
                    VirtualKeyCode::Escape |
                    VirtualKeyCode::C => { state.sub_mr = MenuResult::Cancel; },

                    VirtualKeyCode::W |
                    //VirtualKeyCode::Up |
                    //VirtualKeyCode::Numpad8 |
                    VirtualKeyCode::K => { state.sub_selection = max(0, state.sub_selection - 1); }

                    VirtualKeyCode::S |
                    //VirtualKeyCode::Down |
                    //VirtualKeyCode::Numpad2 |
                    VirtualKeyCode::J => { state.sub_selection = min(num_options - 1, state.sub_selection + 1); }                                   
                    VirtualKeyCode::Return |
                    //VirtualKeyCode::NumpadEnter |
                    VirtualKeyCode::E => {
                        state.sub_mr = MenuResult::Selected;
                        state.result = Some( (menuable.options[state.sub_selection as usize].0, state.e) );
                    }
                    _ => {}
                }
            }

            //Draw the curr-selction arrow.
            ctx.print_color(x, state.pos.1 + state.sub_selection as i32 + 1,
                RGB::named(bracket_lib::prelude::MAGENTA), RGB::named(bracket_lib::prelude::BLACK), "->");

        } else { state.sub_mr = MenuResult::Cancel; }
    } else { state.sub_mr = MenuResult::Cancel; }

    return state;
}

pub fn enable_cursor_control(ecs: &mut World, ctx: &mut Rltk) {
    let input = INPUT.lock();
    let mut movement = 1;
    if input.is_key_pressed(VirtualKeyCode::LShift) { movement = 3 };

    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::K => try_move_cursor((0, -movement), ecs),
            VirtualKeyCode::H => try_move_cursor((-movement, 0), ecs),
            VirtualKeyCode::J => try_move_cursor((0, movement), ecs),
            VirtualKeyCode::L => try_move_cursor((movement, 0), ecs),
            VirtualKeyCode::P => cursor_to_player(ecs),
            _ => {}
        }
    };
}

fn try_move_cursor(delta: (i32, i32), ecs: &mut World) {
    let mut move_cursor_to_player = false;
    
    {
        let map = ecs.fetch::<Map>();
        let mut cursor = ecs.fetch_mut::<Cursor>();

        if !cursor.active {//Is set to false in main.rs
            cursor.active = true;
            move_cursor_to_player = true;
        }

        if cursor.x + delta.0 < 1 || cursor.x + delta.0 > map.width-1 ||
        cursor.y + delta.1 < 1 || cursor.y + delta.1 > map.height-1 {return;}

        cursor.x += delta.0;
        cursor.y += delta.1;
    }
    
    if move_cursor_to_player {
        cursor_to_player(ecs);
    }
}

fn cursor_to_player(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let mut cursor = ecs.fetch_mut::<Cursor>();

    cursor.active = true;
    cursor.x = player_pos.x;
    cursor.y = player_pos.y;
}

pub fn target_selection_mode(ecs: &mut World, ctx: &mut Rltk, range: i32) -> (MenuResult, Option<Point>) {
    
    enable_cursor_control(ecs, ctx); 

    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(ecs, ctx);
    let player_ent = ecs.fetch::<Entity>();
    let player_pos = ecs.fetch::<Point>();
    let cursor = ecs.fetch::<Cursor>();
    let viewsheds = ecs.read_storage::<Viewshed>();

    //Highlight targetable cells
    let mut targetable_cells = Vec::new();
    let vs = viewsheds.get(*player_ent); 
    if let Some(viewshed) = vs {
        for idx in viewshed.visible_tiles.iter() {
            let distance = bracket_lib::prelude::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                if screen_x > 1 && screen_x < (max_x - min_x) - 1 &&
                                screen_y > 1 && screen_y < (max_y - min_y) -1 {
                    
                    ctx.set_bg(screen_x, screen_y, RGB::named(bracket_lib::prelude::TEAL));
                    targetable_cells.push(idx);
                }
            }
        }
    } else {
        let mut log = ecs.fetch_mut::<GameLog>();
        log.entries.push("Cannot target while blind!".to_string());
        return (MenuResult::Cancel, None);
    }

    let mut valid_target = false;
    for idx in targetable_cells.iter() {
         if idx.x == cursor.x && idx.y == cursor.y { valid_target = true; }       
    }

    if valid_target {
        ctx.set_bg(cursor.x - min_x, cursor.y - min_y, RGB::named(bracket_lib::prelude::LIME_GREEN));
    } else {
        ctx.set_bg(cursor.x - min_x, cursor.y - min_y, RGB::named(bracket_lib::prelude::RED));
    }

    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Escape |
            VirtualKeyCode::C => {
                return ( MenuResult::Cancel, None ); }

            VirtualKeyCode::E |
            VirtualKeyCode::Return |
            VirtualKeyCode::NumpadEnter => {
                if valid_target {
                    return ( MenuResult::Selected, Some(Point::new(cursor.x, cursor.y)) ); 
                } else {
                    return ( MenuResult::Continue, None );
                }
            }
            _ => {}
        }
    }

    return (MenuResult::Continue, None);
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk, tooltips: bool) {
    let map = ecs.fetch::<Map>();

    let depth = format!("Depth: {}", map.depth);
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(ecs, ctx);
    let screen_w = max_x - min_x;
    let screen_h = max_y - min_y;

    ctx.draw_box(0, screen_h - 7, screen_w - 1, 6, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK));
    ctx.print_color(2, screen_h - 7, RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), &depth);

    draw_cursor(ecs, ctx);
    draw_stats(ecs, ctx, screen_h, depth.len() as i32);
    draw_tooltips(ecs, ctx, tooltips);
    draw_log(ecs, ctx, screen_h);
}

fn draw_stats(ecs: &World, ctx: &mut Rltk, screen_h: i32, depth_len: i32) {
    let stats = ecs.read_storage::<Stats>();
    let players = ecs.read_storage::<Player>();

    for (_player, stats) in (&players, &stats).join() {
        let mut hp_bar = "".to_string();

        for _ in 0..stats.hp { hp_bar.push_str("♥"); }
        for _ in stats.hp..stats.max_hp { hp_bar.push_str("."); }
        let health = format!("[{}]", &hp_bar);

        ctx.print_color(depth_len + 3, screen_h - 7,
                        RGB::named(bracket_lib::prelude::RED), RGB::named(bracket_lib::prelude::BLACK), &health);
        /*
        let mut fp_bar = "".to_string();
        let mut mp_bar = "".to_string();

        for _ in 0..stats.fp { fp_bar.push_str(">"); }
        for _ in stats.fp..stats.max_fp { fp_bar.push_str("."); }
        let fatigue = format!("[{}]", &fp_bar);

        for _ in 0..stats.mp { mp_bar.push_str("*"); }
        for _ in stats.mp..stats.max_mp { mp_bar.push_str("."); }
        let mana = format!("[{}]", &mp_bar);

        ctx.print_color(depth_len + 6 + stats.max_hp, screen_h - 7,
                        RGB::named(bracket_lib::prelude::GREEN), RGB::named(bracket_lib::prelude::BLACK), &fatigue);
        ctx.print_color(depth_len + 9 + stats.max_hp + stats.max_fp, screen_h - 7,
                        RGB::named(bracket_lib::prelude::BLUE), RGB::named(bracket_lib::prelude::BLACK), &mana);*/
    }
}

fn draw_log(ecs: &World, ctx: &mut Rltk, screen_h: i32) {
    let log = ecs.fetch::<GameLog>();

    let mut y = screen_h - 2;
    let mut i = 0;
    
    for s in log.entries.iter().rev() {
        if i > 4 { break; }
        
        ctx.print_color(2, y,
            RGB::from_u8(255 - (i * 20),255 - (i * 20),255 - (i * 20)),
            RGB::named(bracket_lib::prelude::BLACK), s);
        y -= 1;
        i += 1;
    }
}

fn draw_cursor(ecs: &World, ctx: &mut Rltk) {
    let cursor = ecs.fetch::<Cursor>();
    let (min_x, _, min_y, _) = camera::get_screen_bounds(ecs, ctx);
    if cursor.active { ctx.set_bg(cursor.x - min_x, cursor.y - min_y, RGB::named(bracket_lib::prelude::MAGENTA)); }
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk, global: bool) {

    let map = ecs.fetch::<Map>();
    let cursor = ecs.fetch::<Cursor>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden_storage = ecs.read_storage::<Hidden>();
    let (min_x, _, min_y, _) = camera::get_screen_bounds(ecs, ctx);
    let mut to_tooltip: Vec<(i32, i32, String)> = Vec::new();
    
    if global {
        let runstate = ecs.fetch::<RunState>();
        match *runstate {
            RunState::ShowPlayerMenu { menu_state: _ } => show_player_menu_controls(ctx),
            _ => {
                for (name, pos, _) in (&names, &positions, !&hidden_storage).join() {
                    let idx = map.xy_idx(pos.x, pos.y);
                    if map.visible_tiles[idx] {
                        to_tooltip.push( (pos.x - min_x, pos.y - min_y, name.name.to_string()) );
                    }
                }
            }
        }

    } else if cursor.active == true {
        if cursor.x >= map.width || cursor.y >= map.height ||
                                    cursor.x < 1 || cursor.y < 1 { return; }

        for (name, pos, _) in (&names, &positions, !&hidden_storage).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if pos.x == cursor.x && pos.y == cursor.y && map.visible_tiles[idx] {
                to_tooltip.push( (pos.x - min_x, pos.y - min_y, name.name.to_string()) );
            }
        }       
    }

    if !to_tooltip.is_empty() {
        let terminal_width = ctx.width_pixels / 8;
        let half_width = (terminal_width / 2) as i32;

        for thing in to_tooltip.iter() {
            let x = thing.0;
            let y = thing.1;
            let name = &thing.2;
            let len = thing.2.len() as i32;

            if x >= half_width { 
                ctx.print_color(x - (len + 2), y, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::GREY), name);
                ctx.print_color(x - 2, y, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::GREY), "->");
            } else {
                ctx.print_color(x + 1, y, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::GREY), "<-");
                ctx.print_color(x + 3, y, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::GREY), name);
            } 
        }
    }
}

fn is_adjacent_to_player(ecs: &World, x: i32, y: i32) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let x_dif = (x - player_pos.x).abs();
    let y_dif = (y - player_pos.y).abs();
    
    x_dif <= 1 && y_dif <= 1
}

fn show_player_menu_controls(ctx: &mut Rltk) {
    //let x = ((ctx.width_pixels / 8) as i32 / 3);
    //let y = ((ctx.height_pixels / 8) as i32 / 3);
    let (x, y) = (0, 0);
    ctx.draw_box(x, y, 45, 10, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK));
    ctx.print_color(x + 1, y, RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), "Menu Controls");
    ctx.print(x + 2, y + 2, "Arrow (  ) indicates current selection.");
    ctx.print_color(x + 9, y + 2, RGB::named(bracket_lib::prelude::MAGENTA), RGB::named(bracket_lib::prelude::BLACK), "->");
    ctx.print(x + 2, y + 4, "Up/Down: Navigate Current Page");
    ctx.print(x + 2, y + 5, "Tab: Goto Next Page");
    ctx.print(x + 2, y + 6, "Esc/C: Exit Menu");
    ctx.print(x + 2, y + 7, "Return/Enter/E: Confirm Current Selection");
    ctx.print(x + 2, y + 8, "T: Show/Hide These Controls");
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(bracket_lib::prelude::YELLOW), RGB::named(bracket_lib::prelude::BLACK), "Wizard of the Old Tongue");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(bracket_lib::prelude::MAGENTA), RGB::named(bracket_lib::prelude::BLACK), "New Game");
        } else {
            ctx.print_color_centered(24, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK), "New Game");
        }
        
        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(25, RGB::named(bracket_lib::prelude::MAGENTA), RGB::named(bracket_lib::prelude::BLACK), "Load Game");
            } else {
                ctx.print_color_centered(25, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK), "Load Game");
            }
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, RGB::named(bracket_lib::prelude::MAGENTA), RGB::named(bracket_lib::prelude::BLACK), "Quit");
        } else {
            ctx.print_color_centered(26, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK), "Quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => {
                        return MainMenuResult::NoSelection {selected: MainMenuSelection::Quit}
                    }
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::Quit;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected{ selected : selection },
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

pub fn game_over(ctx: &mut Rltk) -> MenuResult {
    ctx.cls();
    ctx.print_color_centered(12, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK),
        "Your journey has come to its close.");
    ctx.print_color_centered(16, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK),
        "Though history yields not the forgotten, ");
    ctx.print_color_centered(18, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK),
        "it is born of their actions none-the-less.");
    ctx.print_color_centered(22, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK),
        "A new journey awaits.");
    ctx.print_color_centered(28, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK),
        "Will you tread once more upon The Foot of The Mountain?");

    let scale = ctx.get_scale(); //(f32, i32, i32) == (scale, center_x, center_y)
    let center_x = scale.1;

    ctx.print_color(center_x - 5, 32, RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK),
        "Be Reborn");
    ctx.print_color(center_x - 7, 32, RGB::named(bracket_lib::prelude::MAGENTA), RGB::named(bracket_lib::prelude::BLACK),
        "->");

    match ctx.key {
        None => MenuResult::Continue,
        Some(key) => match key {
            VirtualKeyCode::E |
            VirtualKeyCode::Return |
            VirtualKeyCode::NumpadEnter => MenuResult::Selected,
            _ => MenuResult::Continue,
        }
    }
}
