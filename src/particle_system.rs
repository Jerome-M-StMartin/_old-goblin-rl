use specs::prelude::*;
use bracket_lib::prelude::RGB;
use super::{ BTerm, Particle, Position, Renderable, };

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
                        Entities<'a>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Renderable>,
                        WriteStorage<'a, Particle>,
                        WriteExpect<'a, ParticleBuilder>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut positions, mut renderables, mut particles, mut particle_builder) = data;
        for new_particle in particle_builder.requests.iter() {
            
            let p = entities.create();

            positions.insert(p, Position{ x: new_particle.x, y: new_particle.y })
                .expect("Unable to inser position");
            renderables.insert(p, Renderable{ fg: new_particle.fg, bg: new_particle.bg, 
                glyph: new_particle.glyph, render_order: 0 }).expect("Unable to insert renderable");
            particles.insert(p, Particle{ lifetime_ms: new_particle.lifetime })
                .expect("Unable to insert lifetime");
        }

        particle_builder.requests.clear();
    }
}

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: bracket_lib::prelude::FontCharType,
    lifetime: f32
}

pub struct ParticleBuilder {
    requests : Vec<ParticleRequest>
}

impl ParticleBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ParticleBuilder {
        ParticleBuilder{ requests : Vec::new() }
    }

    pub fn request(&mut self, x:i32, y:i32, fg: RGB, bg:RGB, glyph: bracket_lib::prelude::FontCharType, lifetime: f32) {
        self.requests.push(
            ParticleRequest{
                x, y, fg, bg, glyph, lifetime
            }
        );
    }
}

pub fn cull_dead_particles(ecs : &mut World, ctx : &BTerm) {
    let mut dead_particles : Vec<Entity> = Vec::new();
    {
        // Age out particles
        let mut particles = ecs.write_storage::<Particle>();
        let entities = ecs.entities();
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.lifetime_ms -= ctx.frame_time_ms;
            if particle.lifetime_ms < 0.0 {
                dead_particles.push(entity);
            }
        }                    
    }
    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Particle will not die");
    } 
}
