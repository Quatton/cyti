mod plugins;

use bevy::prelude::*;
use bevy_atmosphere::plugin::{AtmosphereCamera, AtmospherePlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use plugins::scene::SceneSetupPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AtmospherePlugin, PanOrbitCameraPlugin))
        .add_plugins(SceneSetupPlugin)
        .add_systems(Startup, spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        AtmosphereCamera::default(),
        PanOrbitCamera::default(),
    ));
}
