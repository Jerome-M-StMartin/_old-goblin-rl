extern crate serde;

mod components;
mod map;
mod player;
mod rect;
mod visibility_system;
mod hostile_ai_system;
mod map_indexing_system;
mod melee_combat_system;
mod damage_system;
mod gui;
mod gamelog;
mod spawner;
mod inventory_system;
mod equip_system;
mod c_menu_system;
mod healing_system;
mod bleed_system;
mod hunger_system;
mod throw_system;
mod light_system;
mod trigger_system;

pub mod particle_system;
pub mod random_table;
pub mod saveload_system;
pub mod map_builders;
pub mod camera;

use rltk::{GameState, Rltk, Point, VirtualKeyCode};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use player::*;
use visibility_system::VisibilitySystem;
use hostile_ai_system::HostileAI;
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use damage_system::DamageSystem;
use inventory_system::ItemCollectionSystem;
use inventory_system::ItemUseSystem;
use inventory_system::ItemDropSystem;
use equip_system::EquipSystem;
use c_menu_system::ContextMenuSystem;
use healing_system::HealingSystem;
use bleed_system::BleedSystem;
use hunger_system::HungerSystem;
use throw_system::ThrowSystem;
use light_system::LightSystem;
use trigger_system::TriggerSystem;

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
    MainMenu { menu_selection: gui::MainMenuSelection },
    MapGeneration,
    NextLevel,
    PreRun,
    PlayerTurn,
    ShowPlayerMenu { menu_state: gui::PlayerMenuState },
    ShowContextMenu { selection: i8, focus: i8 },
    ShowTargeting { range: i32, item: Entity },
    SaveGame,
}

pub struct State {
    pub ecs: World,
    pub tooltips_on: bool,

    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn run_systems(&mut self) {
        let mut context_menu = ContextMenuSystem{};
        context_menu.run_now(&self.ecs);
        let mut mob = HostileAI{};
        mob.run_now(&self.ecs);
        let mut triggers = TriggerSystem{};
        triggers.run_now(&self.ecs);
        let mut items = ItemUseSystem{};
        items.run_now(&self.ecs);
        let mut drop = ItemDropSystem{};
        drop.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut light = LightSystem{};
        light.run_now(&self.ecs);
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut healing = HealingSystem{};
        healing.run_now(&self.ecs);
        let mut bleed = BleedSystem{};
        bleed.run_now(&self.ecs);
        let mut hunger = HungerSystem{};
        hunger.run_now(&self.ecs);
        let mut throw = ThrowSystem{};
        throw.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        let mut pick_up = ItemCollectionSystem{};
        pick_up.run_now(&self.ecs);
        let mut equips = EquipSystem{};
        equips.run_now(&self.ecs);
        let mut particles = particle_system::ParticleSpawnSystem{};
        particles.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn generate_world_map(&mut self, new_depth : i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
        let mut builder = map_builders::random_builder(new_depth, &mut rng, 64, 64);
        builder.build_map(&mut rng);
        std::mem::drop(rng); //drops the borrow on rng & self
        self.mapgen_history = builder.build_data.snapshot_history.clone();
        let player_start;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.build_data.map.clone();
            player_start = builder.build_data.starting_position.as_mut().unwrap().clone();
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
            self.ecs.delete_entity(target).expect("Unable to delete entity");
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
        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog.entries.push("You descend to the next level and take a moment to rest.".to_string());
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
    fn tick(&mut self, ctx: &mut Rltk) { 
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
           
            //reset cursor to inactive state
            match *runstate {
                RunState::AwaitingInput => {}
                RunState::ShowTargeting {range: _, item: _} => {}
                _ => {
                    let mut cursor = self.ecs.fetch_mut::<Cursor>();
                    cursor.active = false;
                }
            }
        }

        ctx.cls(); //clearscreen
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        match newrunstate {
            RunState::MainMenu{..} => {}
            RunState::GameOver => {}
            //RunState::MapGeneration => {}
            _ => {
                
                //draw_map(&self.ecs.fetch::<Map>(), ctx);
                match ctx.key {
                    None => {}
                    Some(key) => match key {
                        VirtualKeyCode::T =>
                            self.tooltips_on = !self.tooltips_on,
                        _ => {}
                    }
                }

                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx, self.tooltips_on);
            }
        }

        match newrunstate {
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                } else {
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
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(&mut self.ecs, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal{..} => newrunstate = RunState::MagicMapReveal { row: 0 },
                    _ => newrunstate = RunState::GameworldTurn
                }
            }
            RunState::GameworldTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowContextMenu { selection, focus } => {
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
            }
            RunState::ShowPlayerMenu { menu_state } => {
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
            }
            RunState::ShowTargeting {range, item} => {
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
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..map.width {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row == map.height - 1 {
                    newrunstate = RunState::GameworldTurn;
                } else {
                    newrunstate = RunState::MagicMapReveal{ row: row + 1 };
                }
            }
            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs); 
                newrunstate = RunState::MainMenu {menu_selection: gui::MainMenuSelection::LoadGame};
            }
            RunState::MainMenu{..} => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection {selected} =>
                        newrunstate = RunState::MainMenu {menu_selection: selected},
                    gui::MainMenuResult::Selected {selected} => {
                        match selected {
                            gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            gui::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                                saveload_system::delete_save();
                            }
                            gui::MainMenuSelection::Quit => {::std::process::exit(0);}
                        }
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::MenuResult::Continue => {}
                    gui::MenuResult::Cancel => {}
                    gui::MenuResult::Selected => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame };
                    }
                }
            }
        }


        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

struct Cursor { 
    pub x: i32, 
    pub y: i32,
    pub active: bool
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Wizard of the Old Tongue")
        .build()?;

    //context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
        tooltips_on: false,

        mapgen_next_state : Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame }),
        mapgen_index : 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0
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
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    gs.ecs.insert(Map::new(1, 64, 64));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MapGeneration{});
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["The Wandering Wood Moves With One's Peripheral Gaze".to_string()]});
    gs.ecs.insert(particle_system::ParticleBuilder::new());
 
    gs.ecs.insert(Cursor { x: 0, y: 0, active: false }); //probably needs attention

    gs.generate_world_map(1);

    rltk::main_loop(context, gs)
}
