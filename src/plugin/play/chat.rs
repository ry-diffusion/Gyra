use std::time::Duration;
use crate::plugin::play::chat_proto;
use crate::state::AppState;
use bevy::a11y::accesskit::TextPosition;
use bevy::prelude::*;
use bevy_cosmic_edit::*;

#[derive(Event, Debug)]
pub struct NewRawChatMessage {
    pub raw_object: String,
}

#[derive(Event, Debug)]
pub struct ChatMessage {
    pub message: String,
}

#[derive(Component, Debug)]
pub struct ChatComponent;

#[derive(Resource, Debug)]
pub struct ChatMaxLines {
    pub max_lines: usize,
}

#[derive(Resource, Debug)]
pub struct ChatBuffer {
    pub buffer: Vec<String>,
    pub shown_buffer: Vec<String>,
}

#[derive(Component)]
pub struct ChatEditor {
    pub timer: Timer
}

#[derive(Component)]
pub struct ChatInputText;

#[derive(Component)]
pub struct ChatInputUI;

pub fn spawn_renderer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let max_lines = 10;
    let chat_font = asset_server.load::<Font>("fonts/chat.otf");

    // TODO: Fix Font
    let attrs = Attrs::new()
        .color(bevy::color::palettes::basic::WHITE.to_cosmic())
        .family(Family::Monospace);

    let text_edit = commands
        .spawn(CosmicEditBundle {
            default_attrs: DefaultAttrs(AttrsOwned::new(attrs)),
            max_lines: MaxLines(1),
            max_chars: MaxChars(100),
            fill_color: CosmicBackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            cursor_color: CursorColor(Color::WHITE),
            text_position: CosmicTextAlign::Left { padding: 12 },

            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(20., 20.)).with_rich_text(
                &mut font_system,
                vec![("", attrs)],
                attrs,
            ),

            ..Default::default()
        })
        .insert(ChatInputText)
        .id();

    commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            style: Style {
                width: Val::Percent(30.0),
                max_height: Val::Percent(50.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..Default::default()
        })
        .with_children(|p| {
            p.spawn(ButtonBundle {
                style: Style {
                    height: Val::Px(40.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            })
            .insert(CosmicSource(text_edit))
            .insert(ChatInputUI);

            p.spawn(TextBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: chat_font.clone(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ),

                style: Style {
                    max_width: Val::Percent(100.0),
                    ..default()
                },

                ..Default::default()
            })
            .insert(ChatEditor {
                timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
            });
        })
        .insert(ChatComponent);
}

fn startup(mut commands: Commands) {
    commands.insert_resource(ChatMaxLines { max_lines: 10 });

    commands.insert_resource(ChatBuffer {
        buffer: Vec::new(),
        shown_buffer: Vec::new(),
    });
}

fn chat_key_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut editor_container: Query<(&mut Visibility, &mut Style), With<ChatInputUI>>,
    mut editor: Query<&mut CosmicEditor, With<ChatInputText>>,
    mut source: Query<&CosmicSource, With<ChatInputUI>>,
    mut focused_widget: ResMut<FocusedWidget>,
    mut chat_writer: EventWriter<ChatMessage>,
) {
    let mut is_focused = false;
    if let Some(focused) = focused_widget.0 {
        for source in source.iter() {
            if source.0 == focused {
                is_focused = true;
            }
        }
    }

    for (mut visibility, mut style) in &mut editor_container.iter_mut() {
        if keys.just_pressed(KeyCode::Enter) && is_focused {
            info!("Hiding chat editor");
            *visibility = Visibility::Hidden;
            style.height = Val::Px(0.0);
            *focused_widget = FocusedWidget(None);

            for mut editor in &mut editor.iter_mut() {
                let text = editor.with_buffer(|b| b.get_text());

                // no empty messages :O
                if text.is_empty() {
                    break;
                }

                chat_writer.send(ChatMessage { message: text });

                clear_buffer(&mut editor);
            }
        }

        if !is_focused {
            *visibility = Visibility::Hidden;
            style.height = Val::Px(0.0);
        }
    }

    if keys.just_pressed(KeyCode::KeyT) && !is_focused {
        for (mut visibility, mut style) in &mut editor_container.iter_mut() {
            if *visibility == Visibility::Visible {
                info!("Hiding chat editor");
                *visibility = Visibility::Hidden;
                style.height = Val::Px(0.0);
                *focused_widget = FocusedWidget(None);
            } else {
                info!("Showing chat editor");
                *visibility = Visibility::Visible;
                style.height = Val::Px(40.0);

                for source in &mut source.iter_mut() {
                    *focused_widget = FocusedWidget(Some(source.0));
                }

                for mut editor in &mut editor.iter_mut() {
                    clear_buffer(&mut editor);
                }
            }
        }
    }
}

fn clear_buffer(editor: &mut Mut<CosmicEditor>) {
    editor.with_buffer_mut(|b| {
        for line in b.lines.iter_mut() {
            let ending = line.ending();
            let attr_list = line.attrs_list().to_owned();
            line.set_text("", ending, attr_list);
        }
    });

    editor.set_cursor(Cursor::new(0, 0));
}

pub fn handle_buffered_text(
    mut query: Query<(&mut Text, &mut ChatEditor)>,
    time: Res<Time>,
    mut chat_buffer: ResMut<ChatBuffer>,
) {
    for (mut text, mut editor) in query.iter_mut() {
        editor.timer.tick(time.delta());
        if editor.timer.finished() {
            editor.timer.reset();
            
            if !chat_buffer.shown_buffer.is_empty() {
                let dropped = chat_buffer.shown_buffer.remove(0);
                chat_buffer.buffer.push(dropped);
            }
        }

        let mut buffer = String::new();
        for line in chat_buffer.shown_buffer.iter() {
            buffer.push_str(line);
            buffer.push_str("\n");
        }

        text.sections[0].value = buffer;
    }
}

pub fn handle_new_chat_messages(
    mut new_messages: EventReader<NewRawChatMessage>,
    mut chat_buffer: ResMut<ChatBuffer>,
    chat_max_lines: Res<ChatMaxLines>,
) {
    for message in new_messages.read() {
        info!("Trying to parse chat message: {}", message.raw_object);

        match serde_json::from_str::<chat_proto::ChatComponent>(&message.raw_object) {
            Ok(msg) => {
                info!("Parsed chat message: {msg:?}");
                chat_buffer.shown_buffer.push(msg.to_plain_text());

                if chat_buffer.shown_buffer.len() > chat_max_lines.max_lines {
                    let item = chat_buffer.shown_buffer.remove(0);
                    info!("Removing item from chat buffer: {item}");
                    chat_buffer.buffer.push(item);
                }
            }
            Err(e) => {
                info!("Failed to parse chat message: {}", message.raw_object);
                error!("{e:?}");
            }
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_event::<NewRawChatMessage>()
        .add_event::<ChatMessage>()
        .add_systems(OnEnter(AppState::Playing), (startup, spawn_renderer))
        .add_systems(
            Update,
            (
                handle_new_chat_messages.run_if(in_state(AppState::Playing)),
                handle_buffered_text.run_if(in_state(AppState::Playing)),
            ),
        )
        .add_systems(
            PostUpdate,
            chat_key_handler.run_if(in_state(AppState::Playing)),
        );
}
