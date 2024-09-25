use crate::message::ClientMessage;
use crate::state::AppState;
use bevy::core_pipeline::motion_blur::{MotionBlur, MotionBlurBundle};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

#[derive(Debug, Component)]
pub(crate) struct Player;

#[derive(Debug, Component)]
struct WorldModelCamera;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), (startup,))
        .add_systems(Update, movement.run_if(in_state(AppState::Playing)))
        .add_systems(Update, (move_camera.run_if(in_state(AppState::Playing))));
}

fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<&mut Transform, With<Player>>,
    cam: Query<&Transform, (With<WorldModelCamera>, Without<Player>)>,
    mut message_writer: EventWriter<ClientMessage>,
) {
    let mut transform = player.single_mut();
    let cam_transform = cam.single();

    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction += *cam_transform.forward();
    }

    if keys.pressed(KeyCode::KeyS) {
        direction += *cam_transform.back();
    }

    if keys.pressed(KeyCode::KeyA) {
        direction += *cam_transform.left();
    }

    if keys.pressed(KeyCode::KeyD) {
        direction += *cam_transform.right();
    }

    if keys.pressed(KeyCode::Space) {
        direction += *cam_transform.up();
    }

    let new_mvnt = direction.normalize_or_zero() * 2.0;
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
                        far: 15.0,
                        ..default()
                    }
                    .into(),
                    ..default()
                },
                MotionBlurBundle {
                    motion_blur: MotionBlur {
                        shutter_angle: 1.0,
                        samples: 2,
                    },
                    ..default()
                },
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
