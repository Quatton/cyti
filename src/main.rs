use bevy::{gltf::Gltf, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AssetLoaderState {
    #[default]
    Loading,
    Done,
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "cars/taxi.glb")]
    pub taxi: Handle<Gltf>,
}

#[derive(Component)]
struct Following {
    facing: Vec3,
}

#[derive(Component)]
struct CanDie;

#[derive(Component)]
struct Player;

impl Default for Following {
    fn default() -> Self {
        Self {
            facing: Vec3::new(0.0, 0.0, 1.0),
        }
    }
}

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
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Update, (rotate_car, move_car))
        .add_systems(Update, spawn_car_on_c)
        .add_systems(Update, kill_out_of_bounds)
        .add_systems(Update, respawn_player_if_not_exists)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    // circular base
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(30.0, 0.1)),
            material: materials.add(Color::WHITE),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cylinder(0.1, 30.0),
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 15.0, 45.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

fn rotate_car(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&CanDie, &mut Transform)>,
) {
    for (_, mut transform) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            // oneway.facing = Quat::from_rotation_y(-0.1) * oneway.facing;
            transform.rotation = Quat::from_rotation_y(-0.1) * transform.rotation;
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            // oneway.facing = Quat::from_rotation_y(0.1) * oneway.facing;
            transform.rotation = Quat::from_rotation_y(0.1) * transform.rotation;
        }

        let cur = transform.rotation;

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            // oneway.facing = Quat::from_rotation_y(0.1) * oneway.facing;
            transform.translation += cur * Vec3::new(0.0, 0.0, 0.5);
        }

        if keyboard_input.pressed(KeyCode::Space) {
            // oneway.facing = Quat::from_rotation_y(0.1) * oneway.facing;
            transform.translation += Vec3::new(0.0, 0.1, 0.0);

            // rotate the x and then remove the y component
            let relative_x = (cur * Vec3::X).reject_from_normalized(Vec3::Y);

            // rotate the car on the relative x axis
            transform.rotation = Quat::from_axis_angle(relative_x, -0.1) * transform.rotation;
        }
    }
}

fn spawn_car_on_c(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    commands: Commands,
    assets: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        spawn_car(commands, assets, false);
    }
}

fn spawn_car(mut commands: Commands, assets: Res<AssetServer>, as_player: bool) {
    let mut binding = commands.spawn((
        SceneBundle {
            scene: assets.load("cars/taxi.glb#Scene0"),
            transform: Transform::from_xyz(0.0, 10.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        CanDie,
    ));

    let cmd = binding.with_children(|p| {
        let mut cmd = p.spawn((
            Collider::cuboid(0.5, 0.5, 1.0),
            Transform::from_xyz(0.0, 0.5, 0.0),
            Restitution::coefficient(0.5),
        ));

        if as_player {
            cmd.insert(ColliderMassProperties::Density(20.0));
        } else {
            cmd.insert(ColliderMassProperties::Density(0.5));
        }

        p.spawn(PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                radius: 10.0,
                intensity: 100000.0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.5, 2.0),
            ..default()
        });
    });

    if as_player {
        cmd.insert(Player);
    } else {
        cmd.insert(Following::default());
    }
}

fn move_car(mut query: Query<(&Following, &mut Transform)>, time: Res<Time>) {
    for (_, mut transform) in query.iter_mut() {
        let rot = transform.rotation;
        transform.translation += rot * Vec3::new(0.0, 0.0, 5.0) * time.delta_seconds();
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

fn respawn_player_if_not_exists(
    commands: Commands,
    assets: Res<AssetServer>,
    query: Query<&Player>,
) {
    if query.iter().count() == 0 {
        spawn_car(commands, assets, true);
    }
}
