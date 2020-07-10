use specs::prelude::*;
use super::{Viewshed, Position, Map, Player};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Player>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y),
                                                        viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width - 1 &&
                                                  p.y >= 0 && p.y < map.height - 1 );
                
                //Light-detecting Viewshed 
                let mut far_viewshed = field_of_view(Point::new(pos.x, pos.y),
                                                 viewshed.range * 10, &*map);
                far_viewshed.retain(|p| p.x >= 0 && p.x < map.width - 1 &&
                                        p.y >= 0 && p.y < map.height - 1 );

                //join of tiles in illuminated_tiles and tiles in far_viewshed
                for i in far_viewshed {
                    let idx = map.xy_idx(i.x, i.y);
                    if map.illuminated_tiles.contains(&idx){ 
                        viewshed.visible_tiles.push(i);
                    }
                }

                // If this is the player, reveal what they can see.
                let p: Option<&Player> = player.get(ent);
                if let Some(_) = p {
                    for t in map.visible_tiles.iter_mut() { *t = false };
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                    }
                }
            }
        }
    }
}
