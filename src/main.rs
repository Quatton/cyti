use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
};
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

    commands
        .spawn((
            SceneBundle {
                scene: assets.load("cars/taxi.glb#Scene0"),
                transform: Transform::from_xyz(0.0, 10.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
        ))
        .with_children(|p| {
            p.spawn((
                Collider::cuboid(0.5, 0.5, 1.0),
                Transform::from_xyz(0.0, 0.5, 0.0),
                Restitution::coefficient(0.5),
            ));

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
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 15.0, 45.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}
