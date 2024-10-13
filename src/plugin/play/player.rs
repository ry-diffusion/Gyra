use crate::components::MainCamera;
use crate::message::ClientMessage;
use crate::plugin::consts::WorldLayer;
use crate::state::AppState;
use bevy::color::palettes::css::WHITE;
use bevy::core_pipeline::motion_blur::{MotionBlur, MotionBlurBundle};
use bevy::input::mouse::MouseMotion;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::view::{
    GpuCulling, NoCpuCulling, NoFrustumCulling, RenderLayers, VisibleEntities,
};

#[derive(Debug, Component)]
pub(crate) struct Player;

#[derive(Debug, Component)]
pub(crate) struct WorldModelCamera;

#[derive(Debug, Component)]
pub struct Aim;

#[derive(Debug, Resource)]
struct PlayerEntity {
    entity: Entity,
}

pub fn plugin(app: &mut App) {
    app.add_plugins(WireframePlugin)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        })
        .add_systems(OnEnter(AppState::Playing), startup)
        .add_systems(OnExit(AppState::Playing), cleanup)
        .add_systems(
            Update,
            (movement, move_camera).run_if(in_state(AppState::Playing)),
        );
}

fn cleanup(mut commands: Commands, player: Res<PlayerEntity>) {
    // for (main_entity, player, entities) in player_q.iter() {
    //     entities.entities.iter().for_each(|(_, entities)| {
    //         entities.iter().for_each(|entity| {
    //             commands.entity(*entity).despawn_recursive();
    //         });
    //     });
    //
    //
    //
    //     commands.entity(main_entity).despawn_recursive();
    //
    //}

    commands.entity(player.entity).despawn_descendants();
    commands.entity(player.entity).despawn_recursive();

    commands.remove_resource::<PlayerEntity>();
}

fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<&mut Transform, With<Player>>,
    mut message_writer: EventWriter<ClientMessage>,
    mut old_position: Local<Vec3>,
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

    let new_movement = direction.normalize_or_zero() * time.delta_seconds() * 20.0;
    transform.translation += new_movement;

    if old_position.distance(transform.translation) > 1.0 {
        message_writer.send(ClientMessage::Moved {
            x: transform.translation.x as _,
            feet_y: (transform.translation.y - 1.62) as _,
            z: transform.translation.z as _,
            on_ground: false,
        });
        *old_position = transform.translation;
    }
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let entity = commands
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
                WorldLayer,
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
                NoFrustumCulling,
                MotionBlurBundle {
                    motion_blur: MotionBlur {
                        shutter_angle: 1.0,
                        samples: 2,
                    },
                    ..default()
                },
            ));
        })
        .id();

    commands.insert_resource(PlayerEntity { entity });
    commands.spawn((
        ImageBundle {
            image: UiImage {
                texture: assets.load("ui/aim.png"),
                ..default()
            },
            style: Style {
                height: Val::Px(16.0),
                width: Val::Px(16.0),
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                left: Val::Percent(50.0),
                bottom: Val::Percent(1.0),
                right: Val::Percent(1.0),
                ..default()
            },
            ..default()
        },
        Aim,
    ));
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
