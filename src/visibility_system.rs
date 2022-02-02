use specs::prelude::*;
use super::{Viewshed, Position, Map, Player, Name, Hidden, BlocksVisibility, gamelog::GameLog};
use bracket_lib::prelude::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        WriteExpect<'a, bracket_lib::prelude::RandomNumberGenerator>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Hidden>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Player>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, BlocksVisibility>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, mut rng, mut log, entities, mut viewshed, mut hidden_storage,
             pos, player, names, blocks_vis) = data;

        map.view_blocked.clear();
        for (pos, _) in (&pos, &blocks_vis).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            map.view_blocked.insert(idx);
        }

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

                        //chance to reveal hidden entities
                        for e in map.tile_content[idx].iter() {
                            let hidden = hidden_storage.get(*e);
                            if let Some(_) = hidden {
                                if rng.roll_dice(1, 50) == 1 {
                                    let name = names.get(*e);
                                    if let Some(name) = name {
                                        log.entries.push(format!("{} spotted!", &name.name));
                                    }
                                    hidden_storage.remove(*e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
