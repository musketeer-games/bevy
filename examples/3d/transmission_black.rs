// This lint usually gives bad advice in the context of Bevy -- hiding complex queries behind
// type aliases tends to obfuscate code while offering no improvement in code cleanliness.
#![allow(clippy::type_complexity)]

use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        tonemapping::Tonemapping,
    },
    pbr::{NotShadowCaster, PointLightShadowMap},
    prelude::*,
    render::view::ColorGrading,
};

#[cfg(not(all(feature = "webgl2", target_arch = "wasm32")))]
use bevy::core_pipeline::experimental::taa::{
    TemporalAntiAliasBundle, TemporalAntiAliasPlugin,
};

use bevy_internal::pbr::NotShadowReceiver;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::AQUAMARINE))
        .insert_resource(PointLightShadowMap { size: 2048 })
        .insert_resource(AmbientLight {
            brightness: 0.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control_system);

    // *Note:* TAA is not _required_ for specular transmission, but
    // it _greatly enhances_ the look of the resulting blur effects.
    // Sadly, it's not available under WebGL.
    #[cfg(not(all(feature = "webgl2", target_arch = "wasm32")))]
    app.insert_resource(Msaa::Off)
        .add_plugins(TemporalAntiAliasPlugin);

    app.run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    commands.spawn(DirectionalLightBundle {
        transform: Transform {
            rotation: Quat::from_axis_angle(Vec3::X, 1.9) * Quat::from_axis_angle(Vec3::Y, PI),
            ..Default::default()
        },
        directional_light: DirectionalLight {
            color:              Color::rgb(0.98, 0.95, 0.82),
            illuminance:        15000.0,
            shadows_enabled:    true,
            shadow_depth_bias:  0.02,
            shadow_normal_bias: 0.6,
        },
        ..Default::default()
    });

    // Floor
    let plane_mesh = meshes.add(shape::Plane::from_size(100.0).into());
    commands.spawn(
        PbrBundle {
            mesh: plane_mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::GREEN,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, -3.0),
            ..default()
        },
    );

    // Cube
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.7 }));
    commands.spawn(
        PbrBundle {
            mesh: cube_mesh,
            material: materials.add(StandardMaterial { base_color: Color::RED, ..default() }),
            transform: Transform::from_xyz(0.25, 0.2, -2.0).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                1.4,
                3.7,
                21.3,
            )),
            ..default()
        }
    );

    // Window
    let quad_mesh = meshes.add(shape::Quad::new(Vec2::splat(4.0)).into());
    commands.spawn((
       NotShadowCaster,
       NotShadowReceiver,
       PbrBundle {
           mesh: quad_mesh,
           material: materials.add(StandardMaterial {
               base_color: Color::WHITE,
               diffuse_transmission: 1.0,
               specular_transmission: 1.0,
               thickness: 1.00,
               ior: 1.4,
               perceptual_roughness: 0.0,
               reflectance: 0.0,
               ..Default::default()
           }),
           transform: Transform {
               translation: Vec3::new(0.25, 1.0, -2.0),
               rotation: Quat::from_euler(
                   EulerRot::XYZ,
                   PI * 1.5,
                   0.0,
                   0.0,
               ),
               ..Default::default()
           },
           ..Default::default()
       },
   ));

    // Camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(1.0, 2.5, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
            color_grading: ColorGrading {
                exposure: -2.0,
                post_saturation: 1.2,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        #[cfg(not(all(feature = "webgl2", target_arch = "wasm32")))]
        TemporalAntiAliasBundle::default(),
        BloomSettings::default(),
    ));
}

#[allow(clippy::too_many_arguments)]
fn camera_control_system(
    mut camera: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
) {

    let mut camera_transform = camera.single_mut();

    let yaw = if input.pressed(KeyCode::Right) {
        time.delta_seconds()
    } else if input.pressed(KeyCode::Left) {
        -time.delta_seconds()
    } else {
        0.0
    };

    let distance_change =
        if input.pressed(KeyCode::Down) && camera_transform.translation.length() < 25.0 {
            time.delta_seconds()
        } else if input.pressed(KeyCode::Up) && camera_transform.translation.length() > 2.0 {
            -time.delta_seconds()
        } else {
            0.0
        };

    camera_transform.translation *= distance_change.exp();

    camera_transform.rotate_around(
        Vec3::ZERO,
        Quat::from_euler(EulerRot::XYZ, 0.0, yaw, 0.0),
    );
}
