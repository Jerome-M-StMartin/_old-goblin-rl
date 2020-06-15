use rltk::{ RGB, Rltk, Point, VirtualKeyCode };
use specs::prelude::*;
use std::cmp::{max, min};
use super::{ Map, Stats, Player, Name, Position, gamelog::GameLog, State, InBackpack,
             Viewshed, RunState, Equipped, Cursor};

#[derive(PartialEq, Clone, Copy)]
pub enum InventoryFocus { Backpack, Equipment }

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected, ChangeFocus }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection {selected: MainMenuSelection},
    Selected {selected: MainMenuSelection}
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Wizard of the Old Tongue");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "New Game");
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "New Game");
        }
        
        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(25, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Load Game");
            } else {
                ctx.print_color_centered(25, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Load Game");
            }
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit");
        } else {
            ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
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

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk, focus: InventoryFocus) ->
                                                (ItemMenuResult, Option<InventoryFocus>, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let items_in_bkpk = (&backpack).join().filter(|item| item.owner == *player_entity).count();
    let items_equipped = (&equipped).join().filter(|item| item.owner == *player_entity).count();
    let items_max = max(items_in_bkpk, items_equipped) as i32;

    let x = 5;
    let menu_width = (ctx.width_pixels / 8) as i32 - (x * 2);
    let max_menu_height = (ctx.height_pixels / 8) as i32 - (x * 2); 
    let mut y = (max_menu_height / 2) - (items_max / 2);

    //draw outer menu box
    ctx.draw_box(x, y-2, menu_width, items_max+3,
        RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(x+3, y-2,
        RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(x+3, min(y+items_max, max_menu_height) + 1,
        RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "TAB to change focus | ESC to Cancel");
    
    //draw focus box
    match focus {
        InventoryFocus::Backpack => {
            ctx.draw_box(x+1, y-1, (menu_width / 2) - 1, (items_in_bkpk + 1) as i32,
                RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK));
            ctx.print_color(x+5, y-1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK),
                "Items in Backpack");
        }
        InventoryFocus::Equipment => {
            ctx.draw_box( (menu_width / 2) + x+1, y-1, (menu_width / 2) - 2, (items_equipped + 1) as i32,
                RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK));
            ctx.print_color( (menu_width / 2) + x+5, y-1,
                RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Equipment");
        }
    }

    let mut selectable: Vec<Entity> = Vec::new();
    let mut j = 0; //for "a, b, c, .." menu selection options.

    //draw unequipped items
    for (entity, name, _in_pack) in (&entities, &names, &backpack).join()
        .filter(|item| item.2.owner == *player_entity) {
        
        ctx.set(x+2, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(x+3, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(x+4, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));
        
        ctx.print(x+6, y, &name.name.to_string());
        selectable.push(entity);
        y += 1;
        j += 1;
    }
        
    j = 0;
    y = (max_menu_height / 2) - (items_max / 2);

    //draw equipped items
    for (entity, name, _equipped) in (&entities, &names, &equipped).join()
        .filter(|item| item.2.owner == *player_entity) {
        
        ctx.set( (menu_width / 2) + x+2, y,
            RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set( (menu_width / 2) + x+3, y,
            RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set( (menu_width / 2) + x+4, y,
            RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));
        ctx.set( (menu_width / 2) + x+5, y,
            RGB::named(rltk::BLACK), RGB::named(rltk::BLACK), rltk::to_cp437(' '));

        ctx.print( (menu_width / 2) + x+6, y, &name.name.to_string());
        selectable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None, None),
        
        Some(key) => {
            match (key, focus) {
                (VirtualKeyCode::Tab, InventoryFocus::Backpack) => {
                    (ItemMenuResult::ChangeFocus, Some(InventoryFocus::Equipment), None) }
               
                (VirtualKeyCode::Tab, InventoryFocus::Equipment) => {
                    (ItemMenuResult::ChangeFocus, Some(InventoryFocus::Backpack), None) }
                
                (VirtualKeyCode::Escape, _) => { (ItemMenuResult::Cancel, None, None) }
                
                (_, InventoryFocus::Backpack) => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < items_in_bkpk as i32 {
                        return (ItemMenuResult::Selected, None, Some(selectable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None, None)
                }
                
                (_, InventoryFocus::Equipment) => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < items_equipped as i32 {
                       return (ItemMenuResult::Selected, None, Some(selectable[items_in_bkpk + selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None, None)
                }
            }
        }
    }
}

//Get rid of this and related shit
pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Drop Which Item?");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut selectable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        selectable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => { 
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(selectable[selection as usize]));
                    }  
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }   
}

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();
    
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Escape => return (ItemMenuResult::Cancel, None),
            _ => {}
        }
    }

    ctx.print_color(5, 0, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Select Target:");

    //Highlight targetable cells
    let mut targetable_cells = Vec::new();
    let visible = viewsheds.get(*player_entity); 
    if let Some(visible) = visible {
        //viewshed exists
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(rltk::BLUE));
                targetable_cells.push(idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    //Show mouse-hovered cell & enable click-select
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in targetable_cells.iter() {
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 { valid_target = true; }
    }

    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_pos.0, mouse_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk, tooltips: bool) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let stats = ecs.read_storage::<Stats>();
    let players = ecs.read_storage::<Player>();
    let log = ecs.fetch::<GameLog>();
    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);

    ctx.print_color(2, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &depth);

    for (_player, stats) in (&players, &stats).join() {
        let mut hp_bar = "".to_string();
        let mut fp_bar = "".to_string();
        let mut mp_bar = "".to_string();

        for _ in 0..stats.hp { hp_bar.push_str("♥"); }
        for _ in stats.hp..stats.max_hp { hp_bar.push_str("."); }
        let health = format!("[{}]", &hp_bar);

        for _ in 0..stats.fp { fp_bar.push_str("○"); }
        for _ in stats.fp..stats.max_fp { fp_bar.push_str("."); }
        let fatigue = format!("[{}]", &fp_bar);

        for _ in 0..stats.mp { mp_bar.push_str("○"); }
        for _ in stats.mp..stats.max_mp { mp_bar.push_str("."); }
        let mana = format!("[{}]", &mp_bar);

        ctx.print_color(depth.len() + 3,
                        43, RGB::named(rltk::RED), RGB::named(rltk::BLACK), &health);
        ctx.print_color(depth.len() + 6 + stats.max_hp as usize,
                        43, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), &fatigue);
        ctx.print_color(depth.len() + 9 + stats.max_hp as usize + stats.max_fp as usize,
                        43, RGB::named(rltk::BLUE), RGB::named(rltk::BLACK), &mana);
    }

    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 { ctx.print(2, y, s); }
        y += 1;
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    
    draw_cursor(ecs, ctx);

    draw_tooltips(ecs, ctx, tooltips);
}

fn draw_cursor(ecs: &World, ctx: &mut Rltk) {
    let cursor = ecs.fetch::<Cursor>();
    if cursor.active { ctx.set_bg(cursor.x, cursor.y, RGB::named(rltk::MAGENTA3)); }
}

pub fn draw_tooltips(ecs: &World, ctx: &mut Rltk, global: bool) {

    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let mouse_pos = ctx.mouse_pos();
    let mut to_tooltip: Vec<(i32, i32, String)> = Vec::new();
    
    if global {
        for (name, pos) in (&names, &positions).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                to_tooltip.push(( pos.x, pos.y, name.name.to_string()) );
            }
        }
    } else {
        if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height { return; }

        for (name, pos) in (&names, &positions).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if pos.x == mouse_pos.0 && pos.y == mouse_pos.1 && map.visible_tiles[idx] {
                to_tooltip.push( (pos.x, pos.y, name.name.to_string()) );
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
                ctx.print_color(x - (len + 2), y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), name);
                ctx.print_color(x - 2, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), "->");
            } else {
                ctx.print_color(x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), "<-");
                ctx.print_color(x + 3, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), name);
            } 
        }
    }
}
