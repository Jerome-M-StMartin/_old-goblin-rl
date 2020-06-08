use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use std::cmp::{max, min};
use super::{Position, Player, Viewshed, State, Map, RunState, Stats, MeleeIntent,
            Item, gamelog::GameLog, PickUpIntent, TileType, Monster};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let stats = ecs.read_storage::<Stats>();
    let entities = ecs.entities();
    let mut melee_intent = ecs.write_storage::<MeleeIntent>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 ||
        pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return; }

        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        
        //bump attack check
        for potential_target in map.tile_content[destination_idx].iter() {
            let target = stats.get(*potential_target);
            if let Some(_target) = target {
                melee_intent.insert(entity, MeleeIntent{target: *potential_target})
                .expect("Add target failed.");
                
                return; //cancel movement
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            viewshed.dirty = true;
            let mut p_pos = ecs.write_resource::<Point>();
            p_pos.x = pos.x;
            p_pos.y = pos.y;
        }
    }
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::StairsDown {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        false
    }
}

fn get_item(ecs: &mut World) {
    let entities = ecs.entities();
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<PickUpIntent>();
            pickup.insert(*player_entity, PickUpIntent {object: item, desired_by: *player_entity})
                .expect("Unable to insert WantToPickUp.");
        }
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {}
                Some(_) => { can_heal = false; }
            }
        }
    }

    if can_heal {
        let mut stats = ecs.write_storage::<Stats>();
        let p_stats = stats.get_mut(*player_entity).unwrap();
        p_stats.hp = max(p_stats.hp, min(p_stats.hp + 1, f32::floor(p_stats.max_hp as f32 / 2.0) as i32));
    }

    RunState::PlayerTurn
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => { return RunState::AwaitingInput }
        Some(key) => match key {
           
            //skip turn; wait
            VirtualKeyCode::Numpad5 => return skip_turn(&mut gs.ecs),
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            //grab item
            VirtualKeyCode::G => get_item(&mut gs.ecs),

            //open inventory
            VirtualKeyCode::I => return RunState::ShowInventory,

            //drop item
            VirtualKeyCode::D => return RunState::ShowDropItem,

            //use stairs
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }

            //save
            VirtualKeyCode::Escape => return RunState::SaveGame,

            //orthogonals
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),
            
            //diagonals
            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::M => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::N => try_move_player(-1, 1, &mut gs.ecs),          

            _ => { return RunState::AwaitingInput }
        },
    }

    RunState::PlayerTurn
}

