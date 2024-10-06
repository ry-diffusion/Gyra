use crate::message::ClientMessage;
use crate::state::AppState;
use bevy::color::palettes::css::WHITE;
use bevy::core_pipeline::motion_blur::{MotionBlur, MotionBlurBundle};
use bevy::input::mouse::MouseMotion;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::view::{GpuCulling, NoCpuCulling, NoFrustumCulling};

#[derive(Debug, Component)]
pub(crate) struct Player;

#[derive(Debug, Component)]
pub(crate) struct WorldModelCamera;

pub fn plugin(app: &mut App) {
    app.add_plugins(WireframePlugin)
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: true,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: WHITE.into(),
        })
        .add_systems(OnEnter(AppState::Playing), startup)
        .add_systems(
            Update,
            (movement, move_camera).run_if(in_state(AppState::Playing)),
        );
}

fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<&mut Transform, With<Player>>,
    mut message_writer: EventWriter<ClientMessage>,
) {
    let mut transform = player.single_mut();

    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction += *transform.forward();
    }

    if keys.pressed(KeyCode::KeyS) {
        direction += *transform.back();
    }

    if keys.pressed(KeyCode::KeyA) {
        direction += *transform.left();
    }

    if keys.pressed(KeyCode::KeyD) {
        direction += *transform.right();
    }

    if keys.pressed(KeyCode::Space) {
        direction += *transform.up();
    }

    if keys.pressed(KeyCode::ShiftLeft) {
        direction += *transform.down();
    }

    let new_mvnt = direction.normalize_or_zero() * time.delta().as_secs_f32() * 5.0;
    transform.translation += new_mvnt;

    message_writer.send(ClientMessage::Moved {
        x: transform.translation.x as _,
        feet_y: (transform.translation.y - 1.62) as _,
        z: transform.translation.z as _,
        on_ground: false,
    });
}

fn startup(mut commands: Commands) {
    commands
        .spawn((
            Player,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                WorldModelCamera,
                Camera3dBundle {
                    camera: Camera {
                        order: 1,
                        ..default()
                    },
                    projection: PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
                GpuCulling,
                NoCpuCulling,
                // NoFrustumCulling,
                //                MotionBlurBundle {
                //                  motion_blur: MotionBlur {
                //                    shutter_angle: 1.0,
                //                        samples: 2,
                //                  },
                //                ..default()
                //          },
            ));
        });
}

fn move_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut player: Query<&mut Transform, With<Player>>,
    mut message_writer: EventWriter<ClientMessage>,
) {
    let mut transform = player.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;

        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        transform.rotate_y(yaw);
        transform.rotate_local_x(pitch);

        message_writer.send(ClientMessage::Look {
            yaw,
            pitch,
            on_ground: false, // TODO
        });
    }
}
