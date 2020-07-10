use specs::prelude::*;
use super::{Position, Map, Lightsource, Flammable, Aflame, Equipped, DamageQueue, DamageAtom};
use rltk::{Point, field_of_view};

pub struct LightSystem {}

impl<'a> System<'a> for LightSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, Map>,
                        WriteStorage<'a, Lightsource>,
                        WriteStorage<'a, Aflame>,
                        ReadStorage<'a, Flammable>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Equipped>,
                        ReadStorage<'a, DamageQueue>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut map, mut lightsources, mut aflame,
             flammables, positions, equipped, damage_queues) = data;
        
        //ignite flammables
        for (ent, _, d_q) in (&entities, &flammables, &damage_queues).join() {
            for dmg in d_q.queue.iter() {
                match dmg {
                    DamageAtom::Thermal(_) => {
                        aflame.insert(ent, Aflame{})
                            .expect("Unable to insert Aflame component.");
                        break;
                    }
                    _ => {}
                }
            }
        }

        //add light to on-fire things
        for (ent, _, _) in (&entities, &aflame, &flammables).join() {
            if let Some(lightsource) = lightsources.get_mut(ent) {
                lightsource.is_lit = true;
            } else {
                lightsources.insert(ent, Lightsource::default())
                    .expect("Unable to insert Lightsource component.");
            }
        }

        //add light to map
        for (ent, lightsource) in (&entities, &mut lightsources).join() {
           
            if !lightsource.is_lit { continue; }

            let mut pos = Position { x: 0, y: 0};

            if let Some(p) = positions.get(ent) {
                pos = *p;
            } else if let Some(e) = equipped.get(ent) {
                let p = positions.get(e.owner).unwrap();
                pos = *p;
            }

            //let lit_circle = BresenhamCircle::new(Point{x: pos.x, y: pos.y}, lightsource.radius);
            let lit_circle = field_of_view(Point::new(pos.x, pos.y), lightsource.radius, &*map);
            for i in lit_circle {
                let idx = map.xy_idx(i.x, i.y);
                map.illuminated_tiles.insert(idx);
            }
        }   
    }
}

