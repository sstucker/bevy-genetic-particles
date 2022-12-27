use bevy::{prelude::*};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::FixedTimestep;
use rand::Rng;

const WINDOW_H: f32 = 1000.;
const WINDOW_W: f32 = 1000.;

pub mod quadtree;
pub use quadtree::*;

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
            force_strength * (1. - repulsion_range + force_range / 2. - x).abs() / (force_range / 2.)
        }
        else {
            -1. * repulsion_strength * (repulsion_range - x) / repulsion_range
        }
    }
}

#[derive(Component)]
struct Cell {
    id: u32
}

#[derive(Component)]
struct Velocity {
    vel: Vec2,
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
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Particles ".to_string() + env!("CARGO_PKG_VERSION"),
                width: WINDOW_W as f32,
                height: WINDOW_H as f32,
                ..default()
            },
            ..default()
        }))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_system(motion_system)
                .with_system(interparticle_force_system)
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>
) {
    commands.spawn(Camera2dBundle::default());

    let window = windows.get_primary().unwrap();

    let mut rng = rand::thread_rng();

    // Spawn some particles
    for i in 1..6400 {

        let h = window.height() / 2.;
        let w = window.width() / 2.;
        let x = rng.gen_range(-w..w as f32);
        let y = rng.gen_range(-h..h as f32);

        commands.spawn(Cell {id: i} )
        .insert( Velocity::new(0., 0.) )
        .insert(Body::new(
            x,  y,
            1.
        ))
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
            transform: Transform::from_translation(Vec3::new(x, y, 0.)),
            ..Default::default()
        });
    }
}

fn motion_system(
    mut query: Query<(&mut Body, &mut Velocity, &mut Transform)>
) {
    for (mut body, mut velocity, mut transform) in query.iter_mut() {
        body.pos = body.pos + velocity.vel;
        transform.translation = body.pos.extend(0.);
        velocity.vel = velocity.vel * 0.9;
    }
}

const R_RANGE: f32 = 10.;
const R_STRENGTH: f32 = 0.05;
const F_RANGE: f32 = 50.;
const F_STRENGTH: f32 = 0.01;

fn interparticle_force_system(
    q_particles: Query<(Entity, &Body)>,
    mut q_velocities: Query<&mut Velocity>
) {
    let mut qtree = CollisionQuadtree::spawn(0., 0., WINDOW_W, WINDOW_H);
    for (entity, body) in q_particles.iter() {
        qtree.insert( EntityBody {entity: entity, position: body.pos, radius: R_RANGE + F_RANGE} )
    }
    let mut colliders: Vec<EntityBody> = Vec::new();
    for (entity, body1) in q_particles.iter() {
        colliders.clear();
        qtree.retrieve(body1.pos, R_RANGE + F_RANGE, &mut colliders);
        if let Ok(mut velocity) = q_velocities.get_mut(entity) {
            for eb in colliders.iter() {
                if eb.entity.index() == entity.index() { continue }
                let distance = body1.pos.distance(eb.position);
                let f = force(
                    distance,
                    R_RANGE,
                    R_STRENGTH,
                    F_RANGE,
                    F_STRENGTH
                );
                let direction = (eb.position - body1.pos).normalize();
                // TODO genetics
                // a = f/m
                // println!("{}", f);
                velocity.vel = velocity.vel + ((direction * f) / body1.mass);
            }
        }
    }
}