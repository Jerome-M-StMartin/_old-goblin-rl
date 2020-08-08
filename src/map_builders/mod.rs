use specs::prelude::*;
use super::{Map, Rect, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER};
mod simple_map;
use simple_map::SimpleMapBuilder;
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


pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;
    fn spawn_entities(&mut self, ecs: &mut World) {
        for ent in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&ent.0, &ent.1));
        }
    }
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 17);
    let mut result: Box<dyn MapBuilder>;

    match builder {
        1 => { result = Box::new(BspDungeonBuilder::new(new_depth)); }
        2 => { result = Box::new(BspInteriorBuilder::new(new_depth)); }
        3 => { result = Box::new(CellularAutomataBuilder::new(new_depth)); }
        4 => { result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)); }
        5 => { result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)); }
        6 => { result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)); }
        7 => { result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)); }
        8 => { result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)); }
        9 => { result = Box::new(MazeBuilder::new(new_depth)); }
        10 => { result = Box::new(DLABuilder::walk_inwards(new_depth)); }
        11 => { result = Box::new(DLABuilder::walk_outwards(new_depth)); }
        12 => { result = Box::new(DLABuilder::central_attractor(new_depth)); }
        13 => { result = Box::new(DLABuilder::insectoid(new_depth)); }
        14 => { result = Box::new(VoronoiCellBuilder::pythagoras(new_depth)); }
        15 => { result = Box::new(VoronoiCellBuilder::manhattan(new_depth)); }
        16 => { result = Box::new(PrefabBuilder::constant(
                new_depth, prefab_builder::prefab_levels::WFC_POPULATED)) },
        _ => { result = Box::new(SimpleMapBuilder::new(new_depth)); }
    }

    if rng.roll_dice(1, 20) == 1 {
        result = Box::new(PrefabBuilder::sectional(new_depth,
                prefab_builder::prefab_sections::UNDERGROUND_FORT, result));
    }

    result = Box::new(PrefabBuilder::vaults(new_depth, result));

    return result;

    /* Example of builder chaining..
    Box::new(
        PrefabBuilder::vaults(
            new_depth,
            Box::new(PrefabBuilder::sectional(
                    new_depth,
                    prefab_builder::prefab_sections::UNDERGROUND_FORT,
                    Box::new(CellularAutomataBuilder::new(new_depth))
            ))
        )
    )*/
}
