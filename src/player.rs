use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use std::cmp::{max, min};
use std::sync::Arc;
use std::any::Any;
use super::{Position, Player, Viewshed, Map, RunState, Stats, MeleeIntent, Cursor,
            Item, gamelog::GameLog, PickUpIntent, TileType, Hostile, Hunger, HungerState,
            JustMoved, user_input::UserInput, gui::command::{Command, Commandable, CommandHistory},
            gui::look_n_feel::Dir};

pub struct PlayerController {
    player: Arc<Player>,
    input: Arc<UserInput>,
    cmd_hist: CommandHistory<PlayerController>,
}

impl PlayerController {

}

//-----------------------------------------------------------------
//------- Command Pattern for UserInput -> PlayerController -------
//-----------------------------------------------------------------
impl Commandable<Player> for Player {
    fn send(&self, cmd: Arc<dyn Command<Player>>) {
        cmd.execute(self);
    }   
}

pub struct MoveCommand {
    move_direction: Dir,
}
impl Command<Player> for MoveCommand {
    fn execute(&self, player: &Player) {
        //try_move_player();
    }
    fn as_any(&self) -> &dyn Any { self }
}

pub struct WaitCommand {}
impl Command<Player> for WaitCommand {
    fn execute(&self, player: &Player) {
        //skip_turn();
    }
    fn as_any(&self) -> &dyn Any { self }
}
//-----------------------------------------------------------------

pub fn player_input(ecs: &mut World, ctx: &mut Rltk) -> RunState {
    let new_runstate : RunState;

    //gui::enable_cursor_control(ecs, ctx);

    new_runstate = match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
          
            VirtualKeyCode::Return => return RunState::ShowContextMenu { selection: 0, focus: 0 },

            //skip turn; wait
            //VirtualKeyCode::Numpad5 |
            VirtualKeyCode::X |
            VirtualKeyCode::Space => skip_turn(ecs),

            //grab item
            VirtualKeyCode::G => get_item(ecs),

            //open backpack/inventory
            VirtualKeyCode::B |
            //VirtualKeyCode::I => return RunState::ShowPlayerMenu { menu_state: PlayerMenuState::default() },

            //use stairs
            VirtualKeyCode::Period => {
                try_next_level(ecs)
            }

            //save
            VirtualKeyCode::Escape => return RunState::SaveGame,

            //orthogonals
            VirtualKeyCode::Left |
            //VirtualKeyCode::Numpad4 |
            VirtualKeyCode::A => try_move_player(-1, 0, ecs),
            VirtualKeyCode::Right |
            //VirtualKeyCode::Numpad6 |
            VirtualKeyCode::D => try_move_player(1, 0, ecs),
            VirtualKeyCode::Up |
            //VirtualKeyCode::Numpad8 |
            VirtualKeyCode::W => try_move_player(0, -1, ecs),
            VirtualKeyCode::Down |
            //VirtualKeyCode::Numpad2 |
            VirtualKeyCode::S => try_move_player(0, 1, ecs),
            
            //diagonals
            //VirtualKeyCode::Numpad9 |
            VirtualKeyCode::E => try_move_player(1, -1, ecs),
            //VirtualKeyCode::Numpad7 |
            VirtualKeyCode::Q => try_move_player(-1, -1, ecs),
            //VirtualKeyCode::Numpad3 |
            VirtualKeyCode::C => try_move_player(1, 1, ecs),
            //VirtualKeyCode::Numpad1 |
            VirtualKeyCode::Z => try_move_player(-1, 1, ecs),          

            _ => return RunState::AwaitingInput,
        },
    };
    
    return new_runstate;
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let player_storage = ecs.read_storage::<Player>();
    let stats = ecs.read_storage::<Stats>();
    let entities = ecs.entities();
    let mut melee_intent = ecs.write_storage::<MeleeIntent>();
    let map = ecs.fetch::<Map>();
    let player = ecs.fetch::<Entity>();
    let mut just_moved_storage = ecs.write_storage::<JustMoved>();

    for (entity, _, pos, viewshed) in
        (&entities, &player_storage, &mut positions, &mut viewsheds).join() {
        
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 ||
        pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return RunState::AwaitingInput; }

        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        
        //bump attack check
        for potential_target in map.tile_content[destination_idx].iter() {
            let target = stats.get(*potential_target);
            if let Some(_target) = target {
                melee_intent.insert(entity, MeleeIntent{target: *potential_target})
                .expect("Add target failed.");
                
                return RunState::PlayerTurn; //cancel movement
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(map.width - 1, max(0, pos.x + delta_x));
            pos.y = min(map.height - 1, max(0, pos.y + delta_y));

            viewshed.dirty = true;
            let mut p_pos = ecs.write_resource::<Point>();
            p_pos.x = pos.x;
            p_pos.y = pos.y;
            
            just_moved_storage.insert(*player, JustMoved{})
                .expect("Unable to insert JustMoved component.");

            //Move Cursor with player
            let mut cursor = ecs.fetch_mut::<Cursor>();
            if cursor.x + delta_x < 1 || cursor.x + delta_x > map.width-1 ||
               cursor.y + delta_y < 1 || cursor.y + delta_y > map.height-1 {return RunState::PlayerTurn;}
            cursor.x += delta_x;
            cursor.y += delta_y;
            return RunState::PlayerTurn;
        }
    }
    
    return RunState::AwaitingInput;
}

fn try_next_level(ecs: &mut World) -> RunState {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);

    if map.tiles[player_idx] == TileType::StairsDown {
        return RunState::NextLevel;
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        return RunState::AwaitingInput;
    }
}

fn get_item(ecs: &mut World) -> RunState {
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
            pickup.insert(*player_entity, PickUpIntent {item, desired_by: *player_entity})
                .expect("Unable to insert WantToPickUp.");
        }
    }

    return RunState::PlayerTurn;
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let hostiles = ecs.read_storage::<Hostile>();
    let hunger_storage = ecs.read_storage::<Hunger>();
    let hunger = hunger_storage.get(*player_entity);
    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = hostiles.get(*entity_id);
            match mob {
                None => {}
                Some(_) => { can_heal = false; }
            }
        }
    }

    if can_heal {
        let mut stats = ecs.write_storage::<Stats>();
        let p_stats = stats.get_mut(*player_entity).unwrap();
        let new_hp = max(p_stats.hp, min(p_stats.hp + 1, f32::floor(p_stats.max_hp as f32 / 2.0) as i32));
        
        if let Some(h) = hunger {
            match h.state {
                HungerState::Famished => p_stats.hp = min(new_hp, max(3, new_hp)),
                HungerState::Starving => {},
                _ => p_stats.hp = new_hp,
            }
        }

    }

    RunState::PlayerTurn
}


