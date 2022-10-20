//! This example demonstrates debugging 3D primitives with wireframes.

use std::f32::consts::PI;

use bevy::{
    pbr::{
        wireframe::{Wireframe, WireframePlugin},
        NotShadowCaster, NotShadowReceiver,
    },
    prelude::*,
    render::{
        primitives::Aabb,
        settings::{WgpuFeatures, WgpuSettings},
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPrimitivesPlugin)
        .add_startup_system(setup)
        .add_system(rotate)
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const X_EXTENT: f32 = 14.;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial::default());

    let shapes = [
        meshes.add(shape::Cube::default().into()),
        meshes.add(shape::Box::default().into()),
        meshes.add(shape::Capsule::default().into()),
        meshes.add(shape::Torus::default().into()),
        meshes.add(shape::Icosphere::default().into()),
        meshes.add(shape::UVSphere::default().into()),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: shape,
                material: debug_material.clone(),
                transform: Transform::from_xyz(
                    -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    2.0,
                    0.0,
                )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Shape,
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 50. }.into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

#[derive(Default, Debug)]
struct DebugPrimitivesPlugin;

impl Plugin for DebugPrimitivesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_plugin(WireframePlugin)
        .init_resource::<DebugPrimitivesConfig>()
        .add_system(add_aabb_debug_primitives)
        .add_system(update_aabb_debug_primitives)
        .add_system(toggle_visibility);
    }
}

#[derive(Resource, Debug)]
pub struct DebugPrimitivesConfig {
    pub is_visible: bool,
}

#[derive(Component, Debug)]
struct DebugPrimitive(Entity);

#[derive(Component, Debug)]
struct DebugPrimitiveParent;

fn add_aabb_debug_primitives(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<DebugPrimitivesConfig>,
    aabb_query: Query<
        Entity,
        (
            With<Aabb>,
            Without<DebugPrimitive>,
            Without<DebugPrimitiveParent>,
        ),
    >,
) {
    for parent_entity in &aabb_query {
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(shape::Cube::default().into()),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    alpha_mode: AlphaMode::Mask(0.5),
                    double_sided: true,
                    cull_mode: None,
                    unlit: true,
                    ..default()
                }),
                ..default()
            })
            .insert(NotShadowCaster)
            .insert(NotShadowReceiver)
            .insert(Visibility {
                is_visible: config.is_visible,
            })
            .insert(Wireframe)
            .insert(DebugPrimitive(parent_entity));

        commands.entity(parent_entity).insert(DebugPrimitiveParent);
    }
}

fn update_aabb_debug_primitives(
    mut commands: Commands,
    config: Res<DebugPrimitivesConfig>,
    mut debug_primitive_query: Query<(Entity, &DebugPrimitive, &mut Transform, &mut Visibility)>,
    aabb_query: Query<(&Aabb, &GlobalTransform), With<DebugPrimitiveParent>>,
) {
    for (debug_primitive_entity, debug_primitive, mut transform, mut visibility) in
        &mut debug_primitive_query
    {
        visibility.is_visible = config.is_visible;

        let (aabb, aabb_transform) = match aabb_query.get(debug_primitive.0) {
            Ok(x) => x,
            Err(_) => {
                commands.entity(debug_primitive_entity).despawn_recursive();
                continue;
            }
        };

        let (aabb_scale, aabb_rotation, aabb_translation) =
            aabb_transform.to_scale_rotation_translation();

        transform.translation = aabb_translation + Vec3::from(aabb.center);
        transform.scale = aabb_scale * Vec3::from(aabb.half_extents * 2.0);
        transform.rotation = aabb_rotation;
    }
}

fn toggle_visibility(
    keyboard_input: Res<Input<KeyCode>>,
    mut config: ResMut<DebugPrimitivesConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        config.is_visible = !config.is_visible;
    }
}

impl Default for DebugPrimitivesConfig {
    fn default() -> Self {
        Self { is_visible: true }
    }
}
