use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::{WindowMode, PresentMode}
};
use bevy::sprite::ColorMaterial;
use bevy::time::FixedTimestep;
use rand::Rng;

const SCALE_FACTOR: f32 = 1.0;

const WINDOW_H: f32 = 1000.;
const WINDOW_W: f32 = 1000.;

const SPEED_LIMIT: f32 = 1000.0;
const FRICTION: f32 = 0.9;

const N_PARTICLES: u32 = 3;

pub mod quadtree;
pub use quadtree::*;

const PI: f32 = 3.14159;

const EPSILON: f32 = 0.000000000000000001;

const INV_255: f32 = 0.00392156862745098;

fn random_u8_array() -> [u8; 255] {
    let mut c = [0; 255];
    let mut rng = rand::thread_rng();
    for i in 0..255 {
        c[i] = rng.gen_range(0..255) as u8;
    }
    c
}

#[inline(always)]
fn u8_to_range(i: u8, minimum: f32, maximum: f32) -> f32 {
    // println!("Decoded u8 {} as {}", i as f32, i as f32 * (INV_255 * maximum - INV_255 * minimum) + minimum);
    (i as f32) * (INV_255 * maximum - INV_255 * minimum) + minimum
}
    
#[inline(always)]
fn range_to_u8(minimum: f32, maximum: f32, v: f32) -> u8 {
    // println!("Encoded f32 {} as {}", v, ((255. * (v - minimum)) / (maximum - minimum)).round() as u8);
    ((255. * (v - minimum)) / (maximum - minimum)).round() as u8
}

#[inline(always)]
fn force(
    x: f32,
    repulsion_range: f32,
    repulsion_strength: f32,
    force_range: f32,
    force_strength: f32
) -> f32 {
    if x > force_range + repulsion_range {
        0.
    }
    else {
        if x > repulsion_range {
            force_strength * (1. - repulsion_range + force_range * 0.5 - x).abs() / (force_range * 0.5)
        }
        else {
            -1. * repulsion_strength * (repulsion_range - x) / repulsion_range
        }
    }
}

const EVOLUTION_PROBABILITY: f32 = 0.2;  // Likelihood that any given gene will increment

const MINIMUM_SIZE: f32 = 40.;

// -- TODO load these from JSON at runtime ----------------

const DIVISION_PROB_MAX: f32 = 0.8;
const DIVISION_PROB_MIN: f32 = 0.7;

const DIVISION_ASYM_MAX: f32 = 0.5;
const DIVISION_ASYM_MIN: f32 = 0.4;

const DIVISION_MIN_SIZE_MAX: f32 = 320.;
const DIVISION_MIN_SIZE_MIN: f32 = 300.;

const REPULSION_RANGE_MAX: f32 = 10.0;
const REPULSION_RANGE_MIN: f32 = 8.0;

const REPULSION_STRENGTH_MAX: f32 = 8.0;
const REPULSION_STRENGTH_MIN: f32 = 78.0;

const FORCE_RANGE_MAX: f32 = 1000.0;
const FORCE_RANGE_MIN: f32 = 50.0;

const FORCE_STRENGTH_MAX: f32 = 0.2;
const FORCE_STRENGTH_MIN: f32 = -0.2;

const EAT_RATE_MAX: f32 = 2.0;
const EAT_RATE_MIN: f32 = -2.0;

// -------------------------------------------------------

struct CellSpawnEvent {
    pos: Vec2,
    vel: Vec2,
    size: f32,
    genome: Genome
}

#[derive(Component, Clone, Copy)]
struct Genome {
    charge: u8,
    division_prob: u8,
    division_asym: u8,
    division_min_size: u8,
    repulsion_range: [u8; 255],
    repulsion_strength: [u8; 255],
    force_range: [u8; 255],
    force_strength: [u8; 255],
    eat_rate: [u8; 255]
}

impl Genome {
    fn new() -> Self {
        Self {
            charge: 1,
            division_prob: range_to_u8(DIVISION_PROB_MIN, DIVISION_PROB_MAX, 0.5 * (DIVISION_PROB_MIN + DIVISION_PROB_MAX)),
            division_asym: range_to_u8(DIVISION_ASYM_MIN, DIVISION_ASYM_MAX, 0.5 * (DIVISION_ASYM_MIN + DIVISION_ASYM_MAX)),
            division_min_size: range_to_u8(DIVISION_MIN_SIZE_MIN, DIVISION_MIN_SIZE_MAX, 0.5 * (DIVISION_MIN_SIZE_MIN + DIVISION_MIN_SIZE_MAX)),
            repulsion_range: [range_to_u8(REPULSION_RANGE_MIN, REPULSION_RANGE_MAX, 0.5 * (REPULSION_RANGE_MIN + REPULSION_RANGE_MAX)); 255],
            repulsion_strength: [range_to_u8(REPULSION_STRENGTH_MIN, REPULSION_STRENGTH_MAX, 0.5 * (REPULSION_STRENGTH_MIN + REPULSION_STRENGTH_MAX)); 255],
            force_range: [range_to_u8(FORCE_RANGE_MIN, FORCE_RANGE_MAX, 0.5 * (FORCE_RANGE_MIN + FORCE_RANGE_MAX)); 255],
            force_strength: [range_to_u8(FORCE_STRENGTH_MIN, FORCE_STRENGTH_MAX, FORCE_STRENGTH_MAX); 255],
            eat_rate: [range_to_u8(FORCE_STRENGTH_MIN, FORCE_STRENGTH_MAX, 0.5 * (FORCE_STRENGTH_MIN + FORCE_STRENGTH_MAX)); 255]
        }
    }

    fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            charge: rng.gen_range(0..255) as u8,
            division_prob: rng.gen_range(0..255) as u8,
            division_asym: rng.gen_range(0..255) as u8,
            division_min_size: rng.gen_range(0..255) as u8,
            repulsion_range: random_u8_array(),
            repulsion_strength: random_u8_array(),
            force_range: random_u8_array(),
            force_strength: random_u8_array(),
            eat_rate: random_u8_array()
        }
    }

    fn mutate_from(genome: &Genome) -> Genome {
        let mut g = genome.clone();
        let mut rng = rand::thread_rng();
        if EVOLUTION_PROBABILITY > rng.gen_range(0.0..1.0) {
            for gene in [&mut g.charge, &mut g.division_prob, &mut g.division_asym, &mut g.division_min_size].iter_mut() {
                if EVOLUTION_PROBABILITY > rng.gen_range(0.0..1.0) {
                    **gene = rng.gen_range(0..255) as u8;
                }
            }
            for gene_arr in [&mut g.repulsion_range, &mut g.repulsion_strength, &mut g.force_range, &mut g.force_strength].iter_mut() {
                if EVOLUTION_PROBABILITY > rng.gen_range(0.0..1.0) {
                    for i in 0..255 {
                        if EVOLUTION_PROBABILITY > rng.gen_range(0.0..1.0) {
                            gene_arr[i] = rng.gen_range(0..255) as u8;
                        }
                    }
                }
            }
        }
        return g
    }

}

#[derive(Component)]
struct Cell {
    id: u32
    // division_timer: Timer
}

impl Cell {
    fn new() -> Self {
        Self {
            id: 0,
            // division_timer = Timer
        }
    }
}

#[derive(Component)]
struct Velocity {
    vel: Vec2,
    growth: f32
}

impl Velocity {
    fn new(dx: f32, dy: f32) -> Self {
        Self {
            vel: Vec2::new(dx, dy),
            growth: 0.0
        }
    }
}

#[derive(Component)]
struct Body {
    pos: Vec2,
    mass: f32
}

impl Body {

    fn new(x: f32, y: f32, mass: f32) -> Self {
        Self { pos: Vec2::new(x, y), mass: mass }
    }

    fn radius(&self) -> f32 {
        (self.mass / PI).sqrt()
    }

}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.21, 0.2)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Particles ".to_string() + env!("CARGO_PKG_VERSION"),
                width: WINDOW_W as f32,
                height: WINDOW_H as f32,
                present_mode: PresentMode::AutoNoVsync,
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            },
            ..default()
        }))
        .add_event::<CellSpawnEvent>()
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                // .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(motion_system)
                .with_system(cell_spawn_system)
                .with_system(intercell_force_system)
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1. / 10.))
                .with_system(division_system)
                .with_system(cell_death_system)
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
    mut ew_spawn: EventWriter<CellSpawnEvent>
) {
    commands.spawn(Camera2dBundle::default());

    let window = windows.get_primary().unwrap();

    let mut rng = rand::thread_rng();

    let g = Genome::random();

    // Spawn some particles
    for i in 0..N_PARTICLES {

        let h = window.height() / 4.;
        let w = window.width() / 4.;
        let x = rng.gen_range(-w..w as f32);
        let y = rng.gen_range(-h..h as f32);

        let x2 = rng.gen_range(-w..w as f32);
        let y2 = rng.gen_range(-h..h as f32);


        ew_spawn.send(
            CellSpawnEvent {
                size: rng.gen_range(2000.0..2400.0 as f32),
                pos: Vec2::new(x2, y2),
                vel: Vec2::new(0., 0.),
                // genome: g
                genome: Genome::random()
            }
        )
    }
}

fn cell_spawn_system(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut reader: EventReader<CellSpawnEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for e in reader.iter() {
        let x = e.pos[0];
        let y = e.pos[1];
        let dx = e.vel[0];
        let dy = e.vel[1];
        let d = 2. * (e.size / PI).sqrt();
        let c = Color::rgb(1.0 - e.genome.charge as f32 / 1024., 0.0 + e.genome.charge as f32 / 255., 1.0 - e.genome.charge as f32 / 1024.);
        commands.spawn( Cell::new() )
        .insert( Velocity::new(dx, dy) )
        .insert( e.genome )
        .insert( Body::new(x, y, e.size) )
        // DEBUG
        // .insert(MaterialMesh2dBundle {
        //     mesh: meshes.add(shape::Circle::new(SCALE_FACTOR * d / 2.).into()).into(),
        //     material: materials.add(c.into()),
        //     transform: Transform::from_translation(Vec3::new(x, y, 0.)),
        //     ..default()
        // });
        // ---
        .insert(SpriteBundle {
            texture: asset_server.load("particle.png"),
            sprite: Sprite {
                color: c,
                custom_size: Some(Vec2::new(d, d)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(x, y, 1.)),
            ..Default::default()
        });
    }
}

fn motion_system(
    mut query: Query<(&mut Body, &mut Velocity, &mut Transform, &mut Sprite)>,
    windows: Res<Windows>
) {
    let window = windows.get_primary().unwrap();
    let w = window.width() * 0.5;
    let h = window.height() * 0.5;
    query.par_for_each_mut(12, |(mut body, mut velocity, mut transform, mut sprite)| {
        let mut rng = rand::thread_rng();
        // if velocity.vel.length_squared() < SPEED_LIMIT {
            // println!("Body at {}, {}", body.pos[0], body.pos[1]);
            if body.pos[0] > w {
                body.pos[0] = w - body.radius();
            }
            else if body.pos[0] < -w {
                body.pos[0] = -w + body.radius();
            }
            if body.pos[1] > h {
                body.pos[1] = h - body.radius();
            }
            else if body.pos[1] < -h {
                body.pos[1] = -h + body.radius();
            }
            // if body.pos.length_squared() < SPEED_LIMIT {
                body.pos = body.pos + velocity.vel;
            // }
            if body.mass > MINIMUM_SIZE {
                body.mass = body.mass + velocity.growth;
            }
            sprite.custom_size = Some(Vec2::new(body.radius() * 2., body.radius() * 2.));
            transform.translation[0] = body.pos[0];
            transform.translation[1] = body.pos[1];
        // }
        velocity.vel = velocity.vel * FRICTION;
        velocity.growth = rng.gen_range(0.0..0.1);
        // velocity.growth = 0.0;
    });
}

fn division_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Body, &Velocity, &Genome, &Cell)>,
    mut ew_spawn: EventWriter<CellSpawnEvent>,
    time: Res<Time>
) {
    let mut rng = rand::thread_rng();
    for (entity, mut body, velocity, genome, cell) in query.iter_mut() {
        if body.mass > u8_to_range(genome.division_min_size, DIVISION_MIN_SIZE_MIN, DIVISION_MIN_SIZE_MAX) {
            if u8_to_range(genome.division_prob, DIVISION_PROB_MIN, DIVISION_PROB_MAX) > rng.gen_range(0.0..1.0) {
                // println!("Cell with mass {} radius {} is dividing!", body.mass, body.radius());
                let div_prop = u8_to_range(genome.division_asym, DIVISION_ASYM_MIN, DIVISION_ASYM_MAX);
                let daughter_angle: f32 = rng.gen_range(0.0..2.*PI);
                ew_spawn.send(
                    CellSpawnEvent {
                        size: div_prop * body.mass,
                        pos: body.pos + Vec2::new(daughter_angle.cos(), daughter_angle.sin()) * body.radius() / 2.,
                        vel: Vec2::from(velocity.vel),
                        genome: Genome::mutate_from(genome)
                    }
                );
                ew_spawn.send(
                    CellSpawnEvent {
                        size: (1.0 - div_prop) * body.mass,
                        pos: body.pos - Vec2::new(daughter_angle.cos(), daughter_angle.sin()) * body.radius() / 2.,
                        vel: Vec2::from(velocity.vel),
                        genome: Genome::mutate_from(genome)
                    }
                );
                commands.entity(entity).despawn();
            }
        }
    }
}

#[inline(always)]
fn cell_forces(
    distance: f32,
    body1: &Body, genome1: &Genome,
    body2: &Body, genome2: &Genome
    ) -> Vec2 {
    let r_range = u8_to_range(genome1.repulsion_range[genome2.charge as usize], REPULSION_RANGE_MIN, REPULSION_RANGE_MAX);
    let f_range = u8_to_range(genome1.force_range[genome2.charge as usize], FORCE_RANGE_MIN, FORCE_RANGE_MAX);
    if distance > r_range + f_range { return Vec2::new(0.0, 0.0) }
    let r_strength = u8_to_range(genome1.repulsion_strength[genome2.charge as usize], REPULSION_STRENGTH_MIN, REPULSION_STRENGTH_MAX);
    let f_strength = u8_to_range(genome1.force_strength[genome2.charge as usize], FORCE_STRENGTH_MIN, FORCE_STRENGTH_MAX);
    let f = force(
        distance,
        r_range + body1.radius(),
        r_strength * (body2.mass / body1.mass),
        f_range,
        f_strength * (body2.mass / body1.mass)
    );
    // a = f/m
    ((body2.pos - body1.pos).normalize() * f) / (body1.mass + EPSILON)
}

fn intercell_force_system(
    mut q_velocity: Query<&mut Velocity>,
    q_cells: Query<(Entity, &Body, &Genome)>
) {
    for ([(e1, body1, g1), (e2, body2, g2)]) in q_cells.iter_combinations() {
        let distance = body1.pos.distance(body2.pos);
        if let Ok(mut velocity) = q_velocity.get_mut(e1) {
            velocity.vel += cell_forces(
                distance,
                body1, g1,
                body2, g2
            );
            if distance < (body1.radius() + body2.radius()) * 2. {
                let net_growth = u8_to_range(g1.eat_rate[g2.charge as usize], EAT_RATE_MIN, EAT_RATE_MAX) - u8_to_range(g2.eat_rate[g1.charge as usize], EAT_RATE_MIN, EAT_RATE_MAX);
                velocity.growth += net_growth;
            } 
        }
        if let Ok(mut velocity) = q_velocity.get_mut(e2) {
            velocity.vel += cell_forces(
                distance,
                body2, g2,
                body1, g1
            );
            if distance < (body1.radius() + body2.radius()) * 2. {
                let net_growth = u8_to_range(g2.eat_rate[g1.charge as usize], EAT_RATE_MIN, EAT_RATE_MAX) - u8_to_range(g1.eat_rate[g2.charge as usize], EAT_RATE_MIN, EAT_RATE_MAX);
                velocity.growth += net_growth;
            } 
        }
    }
}

fn cell_death_system(
    mut commands: Commands,
    query: Query<(Entity, &Body)>
) {
    for (entity, body) in query.iter() {
        if body.mass < MINIMUM_SIZE {
            println!("Cell {} died!", entity.index());
            commands.entity(entity).despawn();
        }
    }
}

// TODO optimize n-body iteration

// fn interparticle_force_system(
//     q_particles: Query<(Entity, &Body)>,
//     mut q_velocities: Query<&mut Velocity>
// ) {
//     let mut qtree = CollisionQuadtree::spawn(0., 0., WINDOW_W, WINDOW_H);
//     for (entity, body) in q_particles.iter() {
//         qtree.insert( EntityBody {entity: entity, position: body.pos, radius: R_RANGE + F_RANGE} )
//     }
//     let mut colliders: Vec<EntityBody> = Vec::new();
//     for (entity, body1) in q_particles.iter() {
//         colliders.clear();
//         qtree.retrieve(body1.pos, R_RANGE + F_RANGE, &mut colliders);
//         if let Ok(mut velocity) = q_velocities.get_mut(entity) {
//             for eb in colliders.iter() {
//                 if eb.entity.index() == entity.index() { continue }
//                 let distance = body1.pos.distance(eb.position);
//                 let f = force(
//                     distance,
//                     R_RANGE,
//                     R_STRENGTH,
//                     F_RANGE,
//                     F_STRENGTH
//                 );
//                 let direction = (eb.position - body1.pos).normalize();
//                 // TODO genetics
//                 // a = f/m
//                 // println!("{}", f);
//                 velocity.vel = velocity.vel + ((direction * f) / body1.mass);
//             }
//         }
//     }
// }