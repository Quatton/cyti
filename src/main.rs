use std::ops::Div;

use bevy::{prelude::*, transform::commands};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (pair_up, move_cubes))
        .run();
}

#[derive(Component)]
struct Cube {
    velocity: Vec3,
}

fn name_generator() -> String {
    let set = ["ma", "ka", "ta", "pa", "ra", "sa", "da", "wa", "la", "ya"];

    let mut name = String::new();
    for _ in 0..3 {
        name.push_str(set.choose(&mut rand::thread_rng()).unwrap());
    }

    format!("{}{}", name[0..1].to_uppercase(), &name[1..])
}

impl Default for Cube {
    fn default() -> Self {
        Cube {
            velocity: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
struct Chasing {
    target: Entity,
}

#[derive(Component)]
struct Evading {
    target: Entity,
}

#[derive(PartialEq)]
enum CubeState {
    Chasing,
    Evading,
}

#[derive(Component)]
struct Interacting {
    behavior: CubeState,
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(30.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    commands.spawn((
        Cube::default(),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Name::new(name_generator()),
    ));

    commands.spawn((
        Cube::default(),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(10.0, 0.5, 0.0),
            ..default()
        },
        Name::new(name_generator()),
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            radius: 4.0,
            intensity: 1000000.0,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 10.0, 0.0),
        ..default()
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 15.0, 45.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

fn pair_up(mut commands: Commands, query: Query<(Entity, &Cube), Without<Interacting>>) {
    let mut it = query.iter_combinations();
    while let Some([(e1, _), (e2, _)]) = it.fetch_next() {
        let mut rng = rand::thread_rng();
        let chance = rng.gen_range(0..2);
        if chance == 0 {
            commands.entity(e1).insert((
                Interacting {
                    behavior: CubeState::Chasing,
                },
                Chasing { target: e2 },
            ));
            commands.entity(e2).insert((
                Interacting {
                    behavior: CubeState::Evading,
                },
                Evading { target: e1 },
            ));
        } else {
            commands.entity(e1).insert((
                Interacting {
                    behavior: CubeState::Evading,
                },
                Evading { target: e2 },
            ));
            commands.entity(e2).insert((
                Interacting {
                    behavior: CubeState::Chasing,
                },
                Chasing { target: e1 },
            ));
        }
    }
}

fn move_cubes(
    time: Res<Time>,
    mut chasers: Query<(&mut Cube, &mut Transform, &Chasing), Without<Evading>>,
    mut evaders: Query<(&mut Cube, &mut Transform, &Evading), Without<Chasing>>,
) {
    for (mut cube, mut transform, chasing) in chasers.iter_mut() {
        let (_, other_transform, _) = evaders.get(chasing.target).unwrap();
        let direction = other_transform.translation - transform.translation;
        let velocity = direction.normalize() * 5.0;
        cube.velocity = velocity;

        if transform.translation.distance(other_transform.translation) > 1.5 {
            transform.translation += cube.velocity * time.delta_seconds();
        }
    }

    for (mut cube, mut transform, evading) in evaders.iter_mut() {
        let (_, other_transform, _) = chasers.get(evading.target).unwrap();
        let direction = other_transform.translation - transform.translation;

        let pi = std::f32::consts::PI;
        let angle = rand::thread_rng().gen_range(pi / 3.0..2.0 * pi / 3.0);
        let velocity = Quat::from_rotation_y(angle) * direction.normalize() * 80.0;

        cube.velocity = velocity;

        if transform.translation.distance(other_transform.translation) > 1.5 {
            transform.translation += cube.velocity * time.delta_seconds();
        }
    }
}
