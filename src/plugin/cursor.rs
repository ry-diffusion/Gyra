use bevy::prelude::*;
use bevy::window::{Cursor, CursorGrabMode, PrimaryWindow};

#[derive(Debug, Resource)]
pub struct CursorState {
    pub is_locked: bool,
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, key_handler)
            .add_systems(Update, state_handler)
            .add_systems(PreUpdate, recenter)
            .add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands) {
    commands.insert_resource(CursorState { is_locked: false });
}

fn key_handler(keys: Res<ButtonInput<KeyCode>>, mut cursor_state: ResMut<CursorState>) {
    if keys.just_pressed(KeyCode::ControlRight) {
        info!("Toggling cursor lock");
        cursor_state.is_locked = !cursor_state.is_locked;
    }
}

fn state_handler(
    cursor_state: ResMut<CursorState>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = q_windows.single_mut();

    if cursor_state.is_changed() {
        if cursor_state.is_locked {
            let cursor = Cursor {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            };

            primary_window.cursor = cursor;
        } else {
            primary_window.cursor.grab_mode = CursorGrabMode::None;
            primary_window.cursor.visible = true;
        }
    }
}

fn recenter(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
    cursor_state: Res<CursorState>,
) {
    if !cursor_state.is_locked {
        return;
    }

    let mut primary_window = q_windows.single_mut();
    let center = Vec2::new(primary_window.width() / 2.0, primary_window.height() / 2.0);
    primary_window.set_cursor_position(Some(center));
}
