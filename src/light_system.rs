use specs::prelude::*;
use super::{Position, Map, Lightsource, Flammable, Equipped, };
use rltk::{Point, BresenhamCircle};

pub struct LightSystem {}

impl<'a> System<'a> for LightSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, Map>,
                        WriteStorage<'a, Lightsource>,
                        ReadStorage<'a, Flammable>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Equipped>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut map, mut lightsources, flammables, positions, equipped) = data;

        for (ent, lightsource) in (&entities, &mut lightsources).join() {
           
            let mut pos = Position { x: 0, y: 0};

            if let Some(p) = positions.get(ent) {
                pos = *p;
            } else if let Some(e) = equipped.get(ent) {
                let p = positions.get(e.owner).unwrap();
                pos = *p;
            }

            let lit_circle = BresenhamCircle::new(Point{x: pos.x, y: pos.y}, lightsource.radius);
            for i in lit_circle {
                let idx = map.xy_idx(i.x, i.y);
                map.illuminated_tiles.insert(idx);
            }
        }   
    }
}

