use bevy::prelude::*;

pub struct SceneSetupPlugin;

impl Plugin for SceneSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scene);
    }
}

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..default()
    },));

    // Add a grassy ground plane
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Cuboid {
            half_size: Vec3::new(100.0, 0.1, 100.0),
        }),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.0, 0.5, 0.0),
            ..default()
        }),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });
}
