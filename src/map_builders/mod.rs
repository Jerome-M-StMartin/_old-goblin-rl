use specs::prelude::*;
use super::{Map, Rect, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod room_based_spawner;
mod room_based_starting_position;
mod room_based_stairs;
mod area_starting_points;
mod cull_unreachable;
mod distant_exit;
mod voronoi_spawning;
mod room_corridors_dogleg;
mod bsp_corridors;
mod room_corner_rounding;
mod room_exploder;
mod room_sorter;
mod room_draw;
mod room_corridors_nearest;
mod room_corridors_lines;
mod room_corridor_spawner;
mod door_placement;
use door_placement::DoorPlacement;
use room_corridor_spawner::CorridorSpawner;
use room_corridors_lines::StraightLineCorridors;
use room_corridors_nearest::NearestCorridors;
use room_draw::RoomDrawer;
use room_sorter::{RoomSorter, RoomSort};
use room_exploder::RoomExploder;
use room_corner_rounding::RoomCornerRounder;
use bsp_corridors::BspCorridors;
use room_corridors_dogleg::DoglegCorridors;
use voronoi_spawning::VoronoiSpawning;
use distant_exit::DistantExit;
use cull_unreachable::CullUnreachable;
use area_starting_points::{AreaStartingPosition, XStart, YStart};
use room_based_spawner::RoomBasedSpawner;
use room_based_starting_position::RoomBasedStartingPosition;
use room_based_stairs::RoomBasedStairs;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod drunkard;
use drunkard::DrunkardsWalkBuilder;
mod maze;
use maze::MazeBuilder;
mod dla;
use dla::DLABuilder;
mod voronoi;
use voronoi::VoronoiCellBuilder;
mod prefab_builder;
use prefab_builder::PrefabBuilder;
mod common;
use common::*;

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut bracket_lib::prelude::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut bracket_lib::prelude::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub snapshot_history: Vec<Map>,
    pub width: i32,
    pub height: i32,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.snapshot_history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(new_depth: i32, width: i32, height: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height),
                starting_position: None,
                rooms: None,
                corridors: None,
                snapshot_history: Vec::new(),
                width,
                height,
            }
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("There can be only one! (starting builder)"),
        };
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng: &mut bracket_lib::prelude::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run map builder chain without starter build."),
            Some(starter) => {
                starter.build_map(rng, &mut self.build_data);
            }
        }

        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for ent in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&ent.0, &ent.1));
        }
    }
}

pub fn random_builder(new_depth: i32, rng: &mut bracket_lib::prelude::RandomNumberGenerator,
                                    width: i32, height: i32) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height);
    let type_roll = rng.roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder)
    }

    if rng.roll_dice(1, 20)==1 {
        builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
    }

    builder.with(DoorPlacement::new());
    builder.with(PrefabBuilder::vaults());

    return builder;
}

fn random_start_position(rng: &mut bracket_lib::prelude::RandomNumberGenerator) -> (XStart, YStart) {
    let x;
    let xroll = rng.roll_dice(1, 3);
    match xroll {
        1 => x = XStart::LEFT,
        2 => x = XStart::CENTER,
        _ => x = XStart::RIGHT
    }

    let y;
    let yroll = rng.roll_dice(1, 3);
    match yroll {
        1 => y = YStart::BOTTOM,
        2 => y = YStart::CENTER,
        _ => y = YStart::TOP
    }

    (x, y)
}

fn random_room_builder(rng: &mut bracket_lib::prelude::RandomNumberGenerator, builder : &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 4);
    match build_roll {
        1..=2 => builder.start_with(SimpleMapBuilder::new()),
        3..=4 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new()) //<--currently unreachable
    }

    // BSP Interior still makes holes in the walls
    if build_roll != 3 {
        // Sort by one of the 5 available algorithms
        let sort_roll = rng.roll_dice(1, 5);
        match sort_roll {
            1 => builder.with(RoomSorter::new(RoomSort::LEFTMOST)),
            2 => builder.with(RoomSorter::new(RoomSort::RIGHTMOST)),
            3 => builder.with(RoomSorter::new(RoomSort::TOPMOST)),
            4 => builder.with(RoomSorter::new(RoomSort::BOTTOMMOST)),
            _ => builder.with(RoomSorter::new(RoomSort::CENTRAL)),
        }

        builder.with(RoomDrawer::new());

        let corridor_roll = rng.roll_dice(1, 4);
        match corridor_roll {
            1 => builder.with(DoglegCorridors::new()),
            2 => builder.with(NearestCorridors::new()),
            3 => builder.with(StraightLineCorridors::new()),
            _ => builder.with(BspCorridors::new())
        }

        let cspawn_roll = rng.roll_dice(1, 2);
        if cspawn_roll == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = rng.roll_dice(1, 6);
        match modifier_roll {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => {}
        }
    }

    let start_roll = rng.roll_dice(1, 2);
    match start_roll {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y));
        }
    }

    let exit_roll = rng.roll_dice(1, 2);
    match exit_roll {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new())
    }

    let spawn_roll = rng.roll_dice(1, 2);
    match spawn_roll {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new())
    }
}

fn random_shape_builder(rng: &mut bracket_lib::prelude::RandomNumberGenerator, builder : &mut BuilderChain) {
    let builder_roll = rng.roll_dice(1, 16);
    match builder_roll {
        1 => builder.start_with(CellularAutomataBuilder::new()),
        2 => builder.start_with(DrunkardsWalkBuilder::open_area()),
        3 => builder.start_with(DrunkardsWalkBuilder::open_halls()),
        4 => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
        5 => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
        6 => builder.start_with(DrunkardsWalkBuilder::fearful_symmetry()),
        7 => builder.start_with(MazeBuilder::new()),
        8 => builder.start_with(DLABuilder::walk_inwards()),
        9 => builder.start_with(DLABuilder::walk_outwards()),
        10 => builder.start_with(DLABuilder::central_attractor()),
        11 => builder.start_with(DLABuilder::insectoid()),
        12 => builder.start_with(VoronoiCellBuilder::pythagoras()),
        13 => builder.start_with(VoronoiCellBuilder::manhattan()),
        _ => builder.start_with(PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED)),
    }

    // Set the start to the center and cull
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());

    // Now set the start to a random starting area
    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    // Setup an exit and spawn mobs
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}
