#[macro_use]
extern crate lazy_static;
extern crate serde;

use std::sync::Arc;

use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

use bracket_lib::prelude::{
    embedded_resource, link_resource, BError, BTerm, BTermBuilder, GameState, Point,
    RandomNumberGenerator, EMBED, KHAKI, WHITE,
};

use command::Commandable;

mod bleed_system;
mod c_menu_system;
mod components;
mod damage_system;
mod equip_system;
mod gui;
mod healing_system;
mod hostile_ai_system;
mod hunger_system;
mod inventory_system;
mod light_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod player;
mod rect;
mod spawner;
mod throw_system;
mod trigger_system;
mod visibility_system;

pub mod camera;
pub mod command;
pub mod map_builders;
pub mod particle_system;
pub mod random_table;
pub mod saveload_system;
pub mod user_input;

//use player::*;
use bleed_system::BleedSystem;
use c_menu_system::ContextMenuSystem;
use damage_system::DamageSystem;
use equip_system::EquipSystem;
use healing_system::HealingSystem;
use hostile_ai_system::HostileAI;
use hunger_system::HungerSystem;
use inventory_system::ItemCollectionSystem;
use inventory_system::ItemDropSystem;
use inventory_system::ItemUseSystem;
use light_system::LightSystem;
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use throw_system::ThrowSystem;
use trigger_system::TriggerSystem;
use visibility_system::VisibilitySystem;
//use gui::drawable::Drawable;

pub use components::*;
pub use map::*;
pub use rect::Rect;

const SHOW_MAPGEN_VISUALIZER: bool = false;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    GameOver,
    GameworldTurn,
    MagicMapReveal { row: i32 },
    //MainMenu { menu_selection: gui::MainMenuSelection }, //OLD
    MainMenu,
    MapGeneration,
    NextLevel,
    PreRun,
    PlayerTurn,
    //ShowPlayerMenu { menu_state: gui::PlayerMenuState }, OLD
    ShowPlayerMenu,
    ShowContextMenu { selection: i8, focus: i8 },
    ShowTargeting { range: i32, item: Entity },
    SaveGame,
}

pub struct State {
    pub ecs: World, //specs World

    user_input: Arc<user_input::UserInput>,
    player_controller: Arc<player::PlayerController>,

    //from-scratch gui crate
    gui: gui::GUI,
    //static_gui_objs: HashMap<String, Arc<dyn Drawable>>, //keeps Rc<things> alive that would otherwise only have Weak<> refs.
    pub tooltips_on: bool, //<-delete after UI integration

    //rltk-based map procgen state - to-be-removed
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn run_systems(&mut self) {
        let mut context_menu = ContextMenuSystem {};
        context_menu.run_now(&self.ecs);
        let mut mob = HostileAI {};
        mob.run_now(&self.ecs);
        let mut triggers = TriggerSystem {};
        triggers.run_now(&self.ecs);
        let mut items = ItemUseSystem {};
        items.run_now(&self.ecs);
        let mut drop = ItemDropSystem {};
        drop.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut light = LightSystem {};
        light.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut healing = HealingSystem {};
        healing.run_now(&self.ecs);
        let mut bleed = BleedSystem {};
        bleed.run_now(&self.ecs);
        let mut hunger = HungerSystem {};
        hunger.run_now(&self.ecs);
        let mut throw = ThrowSystem {};
        throw.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut pick_up = ItemCollectionSystem {};
        pick_up.run_now(&self.ecs);
        let mut equips = EquipSystem {};
        equips.run_now(&self.ecs);
        let mut particles = particle_system::ParticleSpawnSystem {};
        particles.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn generate_world_map(&mut self, new_depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let mut rng = self.ecs.write_resource::<RandomNumberGenerator>();
        let mut builder = map_builders::random_builder(new_depth, &mut rng, 64, 64);
        builder.build_map(&mut rng);
        std::mem::drop(rng); //drops the borrow on rng & self
        self.mapgen_history = builder.build_data.snapshot_history.clone();
        let player_start;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.build_data.map.clone();
            player_start = builder
                .build_data
                .starting_position
                .as_mut()
                .unwrap()
                .clone();
        }

        // Spawn bad guys
        builder.spawn_entities(&mut self.ecs);

        // Place the player and update resources
        let (player_x, player_y) = (player_start.x, player_start.y);
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for (ent, (), (), ()) in (&entities, !&player, !&backpack, !&equipped).join() {
            to_delete.push(ent);
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        // Build a new map and place the player
        let curr_depth;
        {
            let worldmap_resource = self.ecs.fetch::<Map>();
            curr_depth = worldmap_resource.depth;
        }
        self.generate_world_map(curr_depth + 1);

        // Notify the player and give them some health
        let player_entity = self.ecs.fetch::<Entity>();
        let mut logger = gui::gamelog::Logger::new();
        logger.append("You descend to the next level and take a moment to rest.");
        logger.log();
        let mut stats_storage = self.ecs.write_storage::<Stats>();
        let player_stats = stats_storage.get_mut(*player_entity);
        if let Some(stats) = player_stats {
            let new_hp = f32::floor((stats.max_hp - stats.hp) as f32 / 2.0) as i32 + stats.hp;
            stats.hp = new_hp;
        }
    }

    fn game_over_cleanup(&mut self) {
        //Delete All Entities
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        self.generate_world_map(1);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;

            match *runstate {
                RunState::AwaitingInput => {}
                RunState::ShowTargeting { range: _, item: _ } => {}
                _ => {
                    //reset cursor to inactive state
                    /*let mut cursor = self.ecs.fetch_mut::<Cursor>();
                    cursor.active = false;*/
                }
            }
        }

        //Clear both main-map and log consoles.
        //ctx.set_active_console(1);
        //ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();

        particle_system::cull_dead_particles(&mut self.ecs, ctx);
        self.gui.tick(ctx);

        //First, figure out which screen/mode we're on/in. e.g Don't need to draw
        //the map or HUD if we're in the Main Menu or Game Over screens/modes.
        match newrunstate {
            RunState::MainMenu => {}
            RunState::GameOver => {}
            //RunState::MapGeneration => {}
            _ => {
                //draw_map(&self.ecs.fetch::<Map>(), ctx); //Was already commented out, don't remember why. (05/2021)
                /*match ctx.key {
                    None => {}
                    Some(key) => match key {
                        VirtualKeyCode::T =>
                            self.tooltips_on = !self.tooltips_on,
                        _ => {}
                    }
                }*/

                //Updates widget data (Updates VIEW from MVC architecture perspective). ---
                use gui::widget::*;
                let player_ent = self.ecs.fetch::<Entity>();
                let stats_storage = self.ecs.read_storage::<Stats>();
                let player_stats = stats_storage.get(*player_ent);
                let widget_elements = player_stats.unwrap().as_widget_elements();
                store_widget_data("PlayerStats", widget_elements);
                //-------------------------------------------------------------------------

                camera::render_camera(&self.ecs, ctx);
                ctx.set_active_font(0, true);
            }
        }

        //Now match on the runstate again to handle all other runstate-speicific factors.
        match newrunstate {
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                } else {
                    ctx.set_active_console(0);
                    ctx.cls();
                    if self.mapgen_index < self.mapgen_history.len() {
                        camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
                    }
                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > 300.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                        if self.mapgen_index >= self.mapgen_history.len() {
                            newrunstate = self.mapgen_next_state.unwrap();
                        }
                    }
                }
            }
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                self.gui.init_widgets();

                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                //newrunstate = player_input(&mut self.ecs, ctx); //OLD

                //TODO
                /* PlayerController::process() currently loops through its entire
                 * CommandQueue, executing them all sequentially PER CALL,
                 * and I'm not yet sure if this is the behavior I want or not.
                 *
                 * It is IF I also implement some sort of "see-ahead" mode
                 * for the player's turn where they get to pseudo-act in a way that visually
                 * results in gameplay but which is undoable and not committed until they choose
                 * to submit their final turn, which consists of the Commands in the CommandQueue,
                 * in the order they were added to the CommandQueue.*/
                newrunstate = self
                    .player_controller
                    .ecs_process(&mut self.ecs, RunState::AwaitingInput);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        newrunstate = RunState::MagicMapReveal { row: 0 }
                    }
                    _ => newrunstate = RunState::GameworldTurn,
                }
            }
            RunState::GameworldTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            /*RunState::ShowContextMenu { selection, focus } => {
                let result = gui::open_context_menu(&self.ecs, ctx, selection, focus);

                match result.0 {
                    gui::MenuResult::Continue => {
                        newrunstate = RunState::ShowContextMenu { selection: result.2, focus: result.3 };
                    }
                    gui::MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::MenuResult::Selected => {
                        let player = self.ecs.fetch::<Entity>();
                        let ranged_storage = self.ecs.read_storage::<Ranged>();
                        let ranged_item = ranged_storage.get((result.1).unwrap().1);

                        match result.1 {
                            None => {}
                            Some( (menu_option, chosen_ent) ) => match (menu_option, chosen_ent) {
                                (MenuOption::PickUp, _) => {
                                    let mut storage = self.ecs.write_storage::<PickUpIntent>();
                                    storage.insert(*player,
                                        PickUpIntent { item: chosen_ent, desired_by: *player })
                                        .expect("Unable to insert PickUpIntent.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Use, _) => {
                                    if let Some(r) = ranged_item {
                                        newrunstate =
                                            RunState::ShowTargeting { range: r.range, item: chosen_ent };
                                    } else {
                                        let mut storage = self.ecs.write_storage::<UseItemIntent>();
                                        storage.insert(*player,
                                            UseItemIntent { item: chosen_ent, target: None })
                                            .expect("Unable to insert UseItemIntent.");
                                        newrunstate = RunState::PlayerTurn;
                                    }
                                }
                                (MenuOption::DropIt, _) => {
                                    let mut storage = self.ecs.write_storage::<DropItemIntent>();
                                    storage.insert(*player,
                                        DropItemIntent { item: chosen_ent })
                                        .expect("Unable to insert DropItemIntent.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Equip, _) => {
                                    let mut storage = self.ecs.write_storage::<EquipIntent>();
                                    storage.insert(*player,
                                        EquipIntent { item: chosen_ent })
                                        .expect("Unable to insert EquipIntent.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Unequip, _) => {
                                    let mut storage = self.ecs.write_storage::<UnequipIntent>();
                                    storage.insert(*player,
                                        UnequipIntent { item: chosen_ent })
                                        .expect("Unable to insert UnequipIntent component.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Attack, _) => {
                                    let mut storage = self.ecs.write_storage::<MeleeIntent>();
                                    storage.insert(*player,
                                        MeleeIntent { target: chosen_ent })
                                        .expect("Unable to insert MeleeIntent.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Throw, _) => {
                                    if let Some(r) = ranged_item {
                                        newrunstate =
                                            RunState::ShowTargeting { range: r.range, item: chosen_ent };
                                    }
                                }
                            }
                        }
                    }
                }
            }*/
            /*RunState::ShowPlayerMenu { menu_state } => {
                let out = gui::open_player_menu(&self.ecs, ctx, menu_state);

                match out.mr {
                    gui::MenuResult::Continue => {
                        newrunstate = RunState::ShowPlayerMenu { menu_state: out } }
                    gui::MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::MenuResult::Selected => {
                        let player = self.ecs.fetch::<Entity>();
                        let ranged_storage = self.ecs.read_storage::<Ranged>();
                        let ranged_item = ranged_storage.get(out.result.unwrap().1);

                        match out.result {
                            None => {}
                            Some( (menu_option, chosen_ent) ) => match (menu_option, chosen_ent) {
                                (MenuOption::PickUp, _) => {
                                    let mut storage = self.ecs.write_storage::<PickUpIntent>();
                                    storage.insert(*player,
                                        PickUpIntent { item: chosen_ent, desired_by: *player })
                                        .expect("Unable to insert PickUpIntent.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Use, _) => {
                                    if let Some(r) = ranged_item {
                                        newrunstate =
                                            RunState::ShowTargeting { range: r.range, item: chosen_ent };
                                    } else {
                                        let mut storage = self.ecs.write_storage::<UseItemIntent>();
                                        storage.insert(*player,
                                            UseItemIntent { item: chosen_ent, target: None })
                                            .expect("Unable to insert UseItemIntent.");
                                        newrunstate = RunState::PlayerTurn;
                                    }
                                }
                                (MenuOption::DropIt, _) => {
                                    let mut storage = self.ecs.write_storage::<DropItemIntent>();
                                    storage.insert(*player,
                                        DropItemIntent { item: chosen_ent })
                                        .expect("Unable to insert DropItemIntent.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Equip, _) => {
                                    let mut storage = self.ecs.write_storage::<EquipIntent>();
                                    storage.insert(*player,
                                        EquipIntent { item: chosen_ent })
                                        .expect("Unable to insert EquipIntent.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Unequip, _) => {
                                    let mut storage = self.ecs.write_storage::<UnequipIntent>();
                                    storage.insert(*player,
                                        UnequipIntent { item: chosen_ent })
                                        .expect("Unable to insert UnequipIntent component.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Attack, _) => {
                                    let mut storage = self.ecs.write_storage::<MeleeIntent>();
                                    storage.insert(*player,
                                        MeleeIntent { target: chosen_ent })
                                        .expect("Unable to insert MeleeIntent.");
                                    newrunstate = RunState::PlayerTurn;
                                }
                                (MenuOption::Throw, _) => {
                                    if let Some(r) = ranged_item {
                                        newrunstate =
                                            RunState::ShowTargeting { range: r.range, item: chosen_ent };
                                    }
                                }
                            }
                        }
                    }
                }
            }*/
            /*RunState::ShowTargeting {range, item} => {
                let result = gui::target_selection_mode(&mut self.ecs, ctx, range);
                let useable_storage = self.ecs.read_storage::<Useable>();
                let throwable_storage = self.ecs.read_storage::<Throwable>();
                match result.0 {
                    gui::MenuResult::Continue => {}
                    gui::MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::MenuResult::Selected => {
                        if let Some(_) = useable_storage.get(item) {
                            let mut intent = self.ecs.write_storage::<UseItemIntent>();
                            intent.insert(*self.ecs.fetch::<Entity>(), UseItemIntent {item, target: result.1})
                                .expect("Unable to insert UseItemIntent.");
                            newrunstate = RunState::PlayerTurn;
                        } else if let Some(_) = throwable_storage.get(item) {
                            let mut intent = self.ecs.write_storage::<ThrowIntent>();
                            intent.insert(*self.ecs.fetch::<Entity>(), ThrowIntent {item, target: result.1})
                                .expect("Unable to insert UseItemIntent.");
                            newrunstate = RunState::PlayerTurn;

                        }
                    }
                }
            }*/
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..map.width {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row == map.height - 1 {
                    newrunstate = RunState::GameworldTurn;
                } else {
                    newrunstate = RunState::MagicMapReveal { row: row + 1 };
                }
            }

            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
            }

            /*RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {menu_selection: gui::MainMenuSelection::LoadGame};
            }*/
            RunState::MainMenu => {
                use gui::widget::{main_menu, widget_storage};

                if widget_storage::contains("MainMenu") {
                    match self.user_input.get_focus_selection() {
                        Some(0) => {
                            //New Game
                            newrunstate = RunState::PreRun;
                            widget_storage::rm("MainMenu")
                                .expect("widget_storage::rm(main_menu) failed.");
                        }
                        Some(1) => {
                            //Load Game
                            saveload_system::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                            saveload_system::delete_save(); //death is permanent
                            widget_storage::rm("MainMenu")
                                .expect("widget_storage::rm(main_menu) failed.");
                        }
                        Some(2) => ::std::process::exit(0), //Quit Game
                        _ => {}
                    }
                } else {
                    let id: usize = main_menu::construct(ctx, &self.gui.user_input);
                    self.user_input.set_focus(id);
                }
            }

            RunState::GameOver => {
                use gui::widget::{game_over, widget_storage};

                if widget_storage::contains("GameOver") {
                    match self.user_input.get_focus_selection() {
                        Some(0) => {
                            //New Game
                            self.game_over_cleanup();
                            newrunstate = RunState::PreRun;
                            widget_storage::rm("GameOver")
                                .expect("widget_storage::rm('GameOver') failed.");
                        }
                        Some(1) => ::std::process::exit(0), //Quit Game
                        _ => {}
                    }
                } else {
                    let id: usize = game_over::construct(ctx, &self.gui.user_input);
                    self.user_input.set_focus(id);
                }
            }
            _ => {}
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

/*Needs to be more obviously differentiated from GUI::Cursor. -------------------------- !!!
struct Cursor {
    pub x: i32,
    pub y: i32,
    pub active: bool,
}*/

struct Inventory {
    pub hands: (Option<Entity>, Option<Entity>),
    pub quickbar: Vec<Option<Entity>>,
    pub backpack: Vec<Option<Entity>>,
    pub equipment: Vec<Option<Entity>>,
}

//embedded_resource!(TILE_FONT, "../resources/unicode_16x16.png"); //bracket-lib should doc this
embedded_resource!(TILE_FONT, "../resources/terminal8x8.jpg");

fn main() -> BError {
    //link_resource!(TILE_FONT, "resources/unicode_16x16.png");
    link_resource!(TILE_FONT, "resources/terminal8x8.jpg");

    //let context = BTermBuilder::simple80x50()
    let context = BTermBuilder::simple(80, 50)?
        .with_title("GoblinRL")
        .with_fps_cap(30.0)
        //.with_font("unicode_16x16.png", 16, 16) //to use, de-comment embed/link_resource macros
        .with_font("terminal8x8.jpg", 8, 8)
        //.with_font("vga8x16.png", 8, 16)
        //.with_sparse_console(80, 30, "vga8x16.png")
        .build()?;

    //context.with_post_scanlines(true);

    //----------- initialization of State fields ------------
    let user_input = Arc::new(user_input::UserInput::new());

    let player_controller = player::PlayerController::new(&user_input);

    let gui = gui::GUI::new(&user_input);
    //-------------------------------------------------------

    let mut gs = State {
        ecs: World::new(),
        user_input,
        player_controller,
        gui,
        tooltips_on: false,

        //mapgen_next_state : Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame }), OLD
        mapgen_next_state: Some(RunState::MainMenu),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Hostile>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<Stats>();
    gs.ecs.register::<MeleeIntent>();
    gs.ecs.register::<DamageOnUse>();
    gs.ecs.register::<DamageQueue>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<PickUpIntent>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<UseItemIntent>();
    gs.ecs.register::<DropItemIntent>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AoE>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<Resistances>();
    gs.ecs.register::<Weapon>();
    gs.ecs.register::<EquipIntent>();
    gs.ecs.register::<UnequipIntent>();
    gs.ecs.register::<BasicAttack>();
    gs.ecs.register::<BlocksAttacks>();
    gs.ecs.register::<Menuable>();
    gs.ecs.register::<Creature>();
    gs.ecs.register::<Bleeding>();
    gs.ecs.register::<Healing>();
    gs.ecs.register::<Heals>();
    gs.ecs.register::<Immunities>();
    gs.ecs.register::<Particle>();
    gs.ecs.register::<Hunger>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<Useable>();
    gs.ecs.register::<ThrowIntent>();
    gs.ecs.register::<Throwable>();
    gs.ecs.register::<Flammable>();
    gs.ecs.register::<Aflame>();
    gs.ecs.register::<Lightsource>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<JustMoved>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Info>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(Map::new(1, 64, 64));
    gs.ecs.insert(Point::new(0, 0)); //<-what is this, Player Entity location? idk
    gs.ecs.insert(RandomNumberGenerator::new());
    gs.ecs.insert(RunState::MapGeneration {});
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);

    /*gs.ecs.insert(Cursor {
        x: 0,
        y: 0,
        active: false,
    });*/
    gs.ecs.insert(Inventory {
        hands: (None, None),
        quickbar: Vec::new(),
        backpack: Vec::new(),
        equipment: Vec::new(),
    });

    let mut logger = gui::gamelog::Logger::new();
    logger.append("A most stifling damp chokes the air, the");
    logger.color(KHAKI);
    logger.append("Wandering Waters of Ru'Iakh");
    logger.color(WHITE);
    logger.append("have become close.");
    logger.log();

    gs.generate_world_map(1);
    bracket_lib::prelude::main_loop(context, gs)
}
