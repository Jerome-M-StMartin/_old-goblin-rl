use specs::prelude::*;
use super::{RunState, DamageQueue, Bleeding, DamageAtom, Position, Map};

pub struct BleedSystem {} //Damage over Time System

impl<'a> System<'a> for BleedSystem {
    type SystemData = ( Entities<'a>,
                        ReadExpect<'a, RunState>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Bleeding>,
                        WriteStorage<'a, DamageQueue>,
                        WriteExpect<'a, Map>
                      );

    fn run (&mut self, data: Self::SystemData) {
        let (entities, runstate, pos, bleed_storage, mut damage_queues, mut map) = data;

        if *runstate != RunState::GameworldTurn { return; }

        for (ent, pos, _bleed) in (&entities, &pos, &bleed_storage).join() {
            
            DamageQueue::queue_damage(&mut damage_queues, ent, DamageAtom::Bleed);
            
            let blood_pos = map.xy_idx(pos.x, pos.y);
            map.bloodstains.insert(blood_pos); 
        }
    }
}
