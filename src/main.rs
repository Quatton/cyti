use bevy::{gltf::Gltf, prelude::*, transform};
use bevy_asset_loader::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::{na::base, prelude::*};
use rand::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AssetLoaderState {
    #[default]
    Loading,
    Done,
}

#[derive(Resource)]
struct DecisionTimer(Timer);

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "cars/taxi.glb")]
    #[allow(dead_code)]
    pub taxi: Handle<Gltf>,
}

#[derive(Component)]
struct CanMove {
    base_speed: f32,
    base_turn_speed: f32,
    brake: bool,
}

const RADIUS: f32 = 25.0;

impl Default for CanMove {
    fn default() -> Self {
        Self {
            base_speed: 15.0,
            base_turn_speed: 0.0,
            brake: false,
        }
    }
}

#[derive(Component)]
struct CanDie;

fn main() {
    App::new()
        .init_state::<AssetLoaderState>()
        .add_loading_state(
            LoadingState::new(AssetLoaderState::Loading)
                .continue_to_state(AssetLoaderState::Done)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(AssetLoaderState::Done), setup)
        // .add_systems(Update, spawn_box.run_if(in_state(AssetLoaderState::Done)))
        .add_plugins((DefaultPlugins, AtmospherePlugin))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(DecisionTimer(Timer::from_seconds(
            3.0,
            TimerMode::Repeating,
        )))
        .add_systems(
            Update,
            (
                detect_car_collision,
                move_car,
                if_detect_nothing_go_forward,
                car_decides_tick,
            ),
        )
        .add_systems(Update, spawn_car_on_c)
        .add_systems(Update, kill_out_of_bounds)
        .add_systems(Update, kill_all_cars_on_r)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cylinder::new(30.0, 0.1)),
                material: materials.add(Color::WHITE),
                ..default()
            },
            RigidBody::Fixed,
        ))
        .with_children(|parent| {
            parent.spawn((
                Collider::cylinder(5.0, 30.0),
                Friction::coefficient(0.5),
                Transform::from_xyz(0.0, -5.0, 0.0),
                CollisionGroups::new(
                    Group::from_bits(0b0100).unwrap(),
                    Group::from_bits(0b0001).unwrap(),
                ),
            ));

            // parent.spawn((
            //     Collider::cylinder(5.0, 30.0),
            //     ColliderMassProperties::Mass(0.0),
            //     CollisionGroups::new(
            //         Group::from_bits(0b1000).unwrap(),
            //         Group::from_bits(0b0010).unwrap(),
            //     ),
            // ));
        });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 15.0, 45.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
        AtmosphereCamera::default(),
    ));
}

fn spawn_car_on_c(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    commands: Commands,
    assets: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        let mut rand = rand::thread_rng();

        let random_angle = rand.gen_range(0.0..std::f32::consts::PI * 2.0);

        let pos = Vec3::new(
            RADIUS * random_angle.cos() / 2.0,
            0.0,
            RADIUS * random_angle.sin() / 2.0,
        );

        let rot = Quat::from_rotation_y(-random_angle);

        spawn_car(commands, assets, pos, rot);
    }
}

fn spawn_car(mut commands: Commands, assets: Res<AssetServer>, pos: Vec3, rot: Quat) {
    commands
        .spawn((
            SceneBundle {
                scene: assets.load("cars/taxi.glb#Scene0"),
                transform: Transform {
                    translation: pos,
                    rotation: rot,
                    ..default()
                },
                ..default()
            },
            CanMove::default(),
            RigidBody::Dynamic,
            CanDie,
            Velocity::default(),
            ExternalForce::default(),
        ))
        .with_children(|p| {
            p.spawn((
                Collider::cuboid(0.5, 0.5, 1.0),
                Transform::from_xyz(0.0, 0.5, 0.0),
                Restitution::coefficient(0.2),
                ColliderMassProperties::Density(1.0),
                CollisionGroups::new(
                    Group::from_bits(0b0001).unwrap(),
                    Group::from_bits(0b0111).unwrap(),
                ),
            ));

            p.spawn((
                Collider::cone(5.0, 1.0),
                // rotate the cone so it's facing the same direction as the car
                Transform::from_xyz(0.0, 0.5, 6.5)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                Sensor,
                ColliderMassProperties::Mass(0.0),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(
                    Group::from_bits(0b0010).unwrap(),
                    Group::from_bits(0b1001).unwrap(),
                ),
            ));

            p.spawn(SpotLightBundle {
                transform: Transform::from_xyz(0.0, 1.0, 1.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                ..default()
            });
        });
}

fn move_car(
    mut query: Query<(&mut CanMove, &Transform, &Velocity, &mut ExternalForce)>,
    _time: Res<Time>,
) {
    for (mut cm, transform, vel, mut fce) in query.iter_mut() {
        let rot = transform.rotation;
        let direction = rot.mul_vec3(Vec3::Z).reject_from_normalized(Vec3::Y);
        let acc_dir = vel.linvel.normalize().reject_from_normalized(Vec3::Y);

        if transform.translation.length() > RADIUS - 5.0
            && direction.normalize().dot(transform.translation.normalize()) > -0.5
        {
            cm.brake = true;
            cm.base_speed = 0.0;
            cm.base_turn_speed = 5.0;
        }

        let acc = (if vel.linvel.length() > cm.base_speed {
            -1.0
        } else {
            1.0
        }) * 15.0;

        let base_acc = acc - if cm.brake { 30.0 } else { 0.0 };

        fce.force = if base_acc > 0.0 {
            direction * base_acc
        } else {
            acc_dir * base_acc
        };

        let turn_acc = cm.base_turn_speed * 10.0;

        if vel.angvel.length() > cm.base_turn_speed {
            fce.torque = -vel.angvel.normalize() * turn_acc.abs();
        } else {
            fce.torque = turn_acc * Vec3::Y;
        }
    }
}

fn kill_out_of_bounds(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), With<CanDie>>,
) {
    for (entity, transform) in query.iter_mut() {
        if transform.translation.y < -10.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn detect_car_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut parent_query: Query<&Parent>,
    mut car_query: Query<&mut CanMove>,
    sensor_query: Query<&Sensor>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(a, b, _) => {
                let itself = if sensor_query.get(*a).is_ok() { a } else { b };

                let parent = parent_query.get_mut(*itself).unwrap();

                let car = car_query.get_mut(parent.get());

                // let mut rng = rand::thread_rng();
                if let Ok(mut car) = car {
                    // let direction = transform.rotation.mul_vec3(Vec3::Z);

                    car.base_speed = 0.0;
                    car.base_turn_speed = 2.0;
                }
            }
            _ => {
                // println!("Probably safe?");
            } // CollisionEvent::Stopped(a, b, _) => {
              //     let itself = if sensor_query.get(*a).is_ok() { a } else { b };

              //     let parent = parent_query.get_mut(*itself).unwrap();

              //     let car = car_query.get_mut(parent.get());

              //     if let Ok(mut car) = car {
              //         car.speed = 5.0;
              //     }
              // }
        }
    }
}

fn if_detect_nothing_go_forward(
    mut car_query: Query<(&Children, &mut CanMove)>,
    sensor_query: Query<Entity, With<Sensor>>,
    rapier_context: Res<RapierContext>,
) {
    for (children, mut car) in car_query.iter_mut() {
        if car.base_speed > 0.0 {
            continue;
        }

        let sensor = children
            .iter()
            .find_map(|&c| sensor_query.get(c).ok())
            .unwrap();

        if rapier_context
            .intersection_pairs_with(sensor)
            .any(|(_, _, col)| col)
        {
            continue;
        }

        // let direction = transform.rotation.mul_vec3(Vec3::Z);

        *car = CanMove::default();
    }
}

fn kill_all_cars_on_r(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<Entity, With<CanDie>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn car_decides_tick(
    mut timer: ResMut<DecisionTimer>,
    mut query: Query<(&mut CanMove, &Transform)>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();
    if timer.0.tick(time.delta()).just_finished() {
        for (mut cm, transform) in query.iter_mut() {
            // cm = CanMove::default();
        }
    }
}
