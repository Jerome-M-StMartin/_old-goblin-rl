use specs::prelude::*;
use super::{Map, Position, BlocksTile};

/*April, 2022
 * Writing this note from the future, made a tiny readability edit, otherwise
 * this is original to the "tutorial hell" phase of my self-education of Rust
 * and Roguelike Development. Sometime in 2020 I think.
 *
 * The name "MapIndexingSystem" is not sufficiently descriptive, I'd say.
 * 
 * What it appears to do:
 * - ECS-Fetch all entities with both Position and BlocksTile components.
 * - Update a collection storing blocked tile locations with the Position
 *   data from each such entity.
 * - Then push the entity into a collection storing the entities that live
 *   in each tile. (I think this is a Copy, not a passing of ownership,
 *   as Entities are simply IDs. Also taking ownership of an Entity from
 *   the ECS World would be nonsense, right?)
 *
 * When do non-blocking entities get pushed into tile_content[]?
 */

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a> );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);
            
            //if entity at idx blocks, update blocking list.
            let is_blocker: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_) = is_blocker {
                map.blocked[idx] = true;
            }
        
            map.tile_content[idx].push(entity);
        }
    }
}
