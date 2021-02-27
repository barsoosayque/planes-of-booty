use crate::object::WaterObject;
use bevy::prelude::*;

pub fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // spawn scene
    commands
        // water
        .spawn((WaterObject::builder().build(),))
        // cube
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        })
        // light
        .spawn(LightBundle { transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)), ..Default::default() });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0)).looking_at(Vec3::default(), Vec3::unit_y()),
        ..Default::default()
    });
}
