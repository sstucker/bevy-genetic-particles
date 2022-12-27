use bevy::{
    prelude::*,
    window::{WindowMode, PresentMode}
};
use bevy_tasks::TaskPool;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::FixedTimestep;
use rand::Rng;

const WINDOW_H: f32 = 1000.;
const WINDOW_W: f32 = 1000.;

const N_PARTICLES: u32 = 2000;

pub mod quadtree;
pub use quadtree::*;

const PI: f32 = 3.14159;

const INV_255: f32 = 0.00392156862745098;

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


const R_RANGE: f32 = 32.;
const R_STRENGTH: f32 = 120.;
const F_RANGE: f32 = 300.;
const F_STRENGTH: f32 = 3.;

// -- TODO load these from JSON at runtime ----------------

const DIVISION_RATE_MAX: f32 = 0.25;
const DIVISION_RATE_MIN: f32 = 0.001;

const DIVISION_ASYM_MAX: f32 = 0.5;
const DIVISION_ASYM_MIN: f32 = 0.1;

const DIVISION_MIN_SIZE_MAX: f32 = 10.;
const DIVISION_MIN_SIZE_MIN: f32 = 1.;

const REPULSION_RANGE_MAX: f32 = 100.;
const REPULSION_RANGE_MIN: f32 = 5.;

const REPULSION_STRENGTH_MAX: f32 = 300.;
const REPULSION_STRENGTH_MIN: f32 = 50.;

const FORCE_RANGE_MAX: f32 = 600.;
const FORCE_RANGE_MIN: f32 = 20.;

const FORCE_STRENGTH_MAX: f32 = 10.;
const FORCE_STRENGTH_MIN: f32 = -10.;

// -------------------------------------------------------

#[derive(Component)]
struct Genome {
    charge: usize,
    division_rate: u8,
    division_asym: u8,
    division_min_size: u8,
    repulsion_range: [u8; 255],
    repulsion_strength: [u8; 255],
    force_range: [u8; 255],
    force_strength: [u8; 255]
}

impl Genome {
    fn new() -> Self {
        Self {
            charge: 128,
            division_rate: range_to_u8(DIVISION_RATE_MIN, DIVISION_RATE_MAX, 0.01),
            division_asym: range_to_u8(DIVISION_ASYM_MIN, DIVISION_ASYM_MAX, 0.5),
            division_min_size: range_to_u8(DIVISION_MIN_SIZE_MIN, DIVISION_MIN_SIZE_MAX, 2.),
            repulsion_range: [range_to_u8(REPULSION_RANGE_MIN, REPULSION_RANGE_MAX, R_RANGE); 255],
            repulsion_strength: [range_to_u8(REPULSION_STRENGTH_MIN, REPULSION_STRENGTH_MAX, R_STRENGTH); 255],
            force_range: [range_to_u8(FORCE_RANGE_MIN, FORCE_RANGE_MAX, F_RANGE); 255],
            force_strength: [range_to_u8(FORCE_STRENGTH_MIN, FORCE_STRENGTH_MAX, F_STRENGTH); 255]
        }
    }
}

#[derive(Component)]
struct Cell {
    id: u32
}

#[derive(Component)]
struct Velocity {
    vel: Vec2
}

impl Velocity {
    fn new(dx: f32, dy: f32) -> Self {
        Self { vel: Vec2::new(dx, dy) }
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
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.23)))
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
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_system(motion_system)
                .with_system(intercell_force_system)
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>
) {
    commands.spawn(Camera2dBundle::default());

    let window = windows.get_primary().unwrap();

    let mut rng = rand::thread_rng();

    // Spawn some particles
    for i in 0..N_PARTICLES {

        let h = window.height() / 2.;
        let w = window.width() / 2.;
        let x = rng.gen_range(-w..w as f32);
        let y = rng.gen_range(-h..h as f32);
        
        let size = rng.gen_range(50.0..500.0 as f32);
        let d = 2. * (size / PI).sqrt();

        commands.spawn(Cell {id: i} )
        .insert( Velocity::new(0., 0.) )
        .insert( Genome::new() )
        .insert( Body::new(x, y, size) )
        // Circle
        // .insert(MaterialMesh2dBundle {
        //     mesh: meshes.add(shape::Circle::new(5.).into()).into(),
        //     material: materials.add(ColorMaterial::from(Color::WHITE)),
        //     transform: Transform::from_translation(Vec3::new(x, y, 0.)),
        //     ..default()
        // });
        // Sprite
        .insert(SpriteBundle {
            texture: asset_server.load("particle.png"),
            sprite: Sprite {
                color: Color::rgb(0.7, 0.7, 0.7),
                custom_size: Some(Vec2::new(d, d)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(x, y, 0.)),
            ..Default::default()
        });
    }
}

fn motion_system(
    mut query: Query<(&mut Body, &mut Velocity, &mut Transform)>
) {
    query.par_for_each_mut(12, |(mut body, mut velocity, mut transform)| {
        body.pos = body.pos + velocity.vel;
        transform.translation = body.pos.extend(0.);
        velocity.vel = velocity.vel * 0.9;
    });
}

#[inline(always)]
fn cell_forces(body1: &Body, genome1: &Genome,
               body2: &Body, genome2: &Genome) -> Vec2 {
    let distance = body1.pos.distance(body2.pos);
    let r_range = u8_to_range(genome1.repulsion_range[genome2.charge], REPULSION_RANGE_MIN, REPULSION_RANGE_MAX);
    let f_range = u8_to_range(genome1.force_range[genome2.charge], FORCE_RANGE_MIN, FORCE_RANGE_MAX);
    if distance > r_range + f_range { return Vec2::new(0.0, 0.0) }
    let r_strength = u8_to_range(genome1.repulsion_strength[genome2.charge], REPULSION_STRENGTH_MIN, REPULSION_STRENGTH_MAX);
    let f_strength = u8_to_range(genome1.force_strength[genome2.charge], FORCE_STRENGTH_MIN, FORCE_STRENGTH_MAX);
    let f = force(
        distance,
        r_range,
        r_strength,
        f_range,
        f_strength
    );
    // a = f/m
    ((body2.pos - body1.pos).normalize() * f) / body1.mass
}

fn intercell_force_system(
    mut q_velocity: Query<&mut Velocity>,
    q_cells: Query<(Entity, &Body, &Genome)>
) {
    for ([(e1, body1, g1), (e2, body2, g2)]) in q_cells.iter_combinations() {
        // println!("{} <---> {}", e1.index(), e2.index());
        if let Ok(mut velocity) = q_velocity.get_mut(e1) {
            velocity.vel += cell_forces(
                body1, g1,
                body2, g2
            )
        }
        if let Ok(mut velocity) = q_velocity.get_mut(e2) {
            velocity.vel += cell_forces(
                body2, g2,
                body1, g1
            )
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