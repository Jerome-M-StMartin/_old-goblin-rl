use specs::prelude::*;
use super::{Viewshed, Hostile, Map, Position, MeleeIntent, RunState, Confusion};
use rltk::{Point};

pub struct HostileAI {}

impl<'a> System<'a> for HostileAI { // 'a syntax is var name for a "lifetime"
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Hostile>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, MeleeIntent>,
                        WriteStorage<'a, Confusion> );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, player_entity, runstate, entities,
             mut viewshed, hostile, mut position, mut melee_intent, mut confusion) = data;
       
        if *runstate != RunState::GameworldTurn { return; }

        for (entity, mut viewshed, _hostile, mut pos) in (&entities, &mut viewshed, &hostile, &mut position).join() { 
            let mut can_act = true;

            let is_confused = confusion.get_mut(entity);
            if let Some(confused_creature) = is_confused {
                confused_creature.turns -= 1;
                if confused_creature.turns < 1 {
                    confusion.remove(entity);
                }
                can_act = false;
            }

            if can_act {
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance <= 1.5 {
                    melee_intent.insert(entity, MeleeIntent{ target: *player_entity })
                        .expect("Uname to insert attack.");
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = rltk::a_star_search( //path to player
                        map.xy_idx(pos.x, pos.y),
                        map.xy_idx(player_pos.x, player_pos.y),
                        &*map
                    );

                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}

