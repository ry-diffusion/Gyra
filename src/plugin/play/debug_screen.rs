use std::process::id;

use crate::components::MainCamera;
use crate::plugin::play::player;
use crate::plugin::play::player::WorldModelCamera;
use crate::plugin::play::world::{ActivePlayerChunks, ShownPlayerChunks, WorldChunkData};
use crate::state::AppState;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::pbr::wireframe::WireframeConfig;
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::render::view::VisibleEntities;
use std::time::SystemTime;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind};

#[derive(Resource, Debug)]
pub struct DebugScreenActive;

#[derive(Component)]
struct Menu;

#[derive(Component)]
struct PositionText;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct RenderText;

#[derive(Component)]
struct ChunkText;

#[derive(Component)]
struct CpuText;

#[derive(Component)]
struct MemoryUsageText;

#[derive(Resource)]
struct DiagnosticsTimer {
    timer: Timer,
}

#[derive(Event)]
pub struct DiagnosticReport {
    pub compute: f32,
    pub async_compute: f32,
    pub io: f32,
    pub main: f32,

    // bytes
    pub memory_usage: u64,

    // bytes
    pub avaliable_memory: u64,
}

#[cfg(windows)]
// Blah blah blah, windows specific code
// Why do your house doesn't have windows?
// Cuz the view of the windows is a shit.
thread_local! {
    static __WIN_LAST_TIME: std::cell::RefCell<Option<TimeReport>> = std::cell::RefCell::new(None);
}

pub fn plugin(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, spawn)
        .add_event::<DiagnosticReport>()
        .insert_resource(DiagnosticsTimer {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        })
        .add_systems(
            Update,
            (
                update_diagnostics_text,
                update_diagnostics_values.run_if(resource_exists::<DebugScreenActive>),
                debug_screen_handler,
                update_render_info,
                update_chunk_info,
                update_position_data
                    .run_if(resource_exists::<DebugScreenActive>)
                    .run_if(in_state(AppState::Playing)),
                update_fps.run_if(resource_exists::<DebugScreenActive>),
            ),
        );
}

fn get_all_visible_num(vs: &VisibleEntities) -> usize {
    let mut total = 0;
    for item in vs.entities.values() {
        total += item.len();
    }
    total
}

fn update_chunk_info(
    mut text_q: Query<&mut Text, With<ChunkText>>,
    world_data: Option<Res<WorldChunkData>>,
    active_chunks: Option<Res<ActivePlayerChunks>>,
    shown_active_chunks: Option<Res<ShownPlayerChunks>>,
) {
    if let (Some(world_data), Some(active_chunks), Some(shown)) =
        (world_data, active_chunks, shown_active_chunks)
    {
        let mut text = text_q.single_mut();
        text.sections[1].value = format!(" mem: {}", world_data.loaded_column.keys().len());
        text.sections[2].value = format!(" active: {}", active_chunks.chunks.len());
        text.sections[3].value = format!(" shown: {}", shown.renderized.len());
    }
}

fn update_render_info(
    mut text_q: Query<&mut Text, With<RenderText>>,
    world_camera: Query<&VisibleEntities, With<WorldModelCamera>>,
    main_camera: Query<&VisibleEntities, With<MainCamera>>,
) {
    let main = main_camera.single();
    let mut text = text_q.single_mut();
    text.sections[1].value = format!(" main: {}", get_all_visible_num(main));

    if let Ok(world) = world_camera.get_single() {
        text.sections[2].value = format!(" world: {}", get_all_visible_num(world));
    }
}

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut fps_text: Query<&mut Text, With<FpsText>>) {
    if let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        let mut fps_text = fps_text.single_mut();
        fps_text.sections[1].value = format!("{value:>4.0}");

        fps_text.sections[1].style.color = if value >= 120.0 {
            // Above 120 FPS, use green color
            Color::srgb(0.0, 1.0, 0.0)
        } else if value >= 60.0 {
            // Between 60-120 FPS, gradually transition from yellow to green
            Color::srgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
        } else if value >= 30.0 {
            // Between 30-60 FPS, gradually transition from red to yellow
            Color::srgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
        } else {
            // Below 30 FPS, use red color
            Color::srgb(1.0, 0.0, 0.0)
        }
    }
}

fn update_position_data(
    mut position_text: Query<&mut Text, With<PositionText>>,
    player_transform: Query<&Transform, With<player::Player>>,
) {
    let mut position_text = position_text.single_mut();

    let transform = player_transform.single();
    let pos = transform.translation;
    let rot = transform.rotation;
    position_text.sections[1].value =
        format!(
        " map: {:>2.0}/{:>2.0}/{:>2.0}, rot: {:>2.0}/{:>2.0}/{:>2.0}/{:>2.0}, chk: {:>2.0}/{:>2.0}",
        pos.x, pos.y, pos.z, rot.x, rot.y, rot.z, rot.w,
        (pos.x / 16.0).floor(), (pos.z / 16.0).floor()
    );
}

fn spawn(mut commands: Commands, rd: Res<RenderAdapterInfo>) {
    commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            style: Style {
                max_width: Val::Percent(30.0),
                max_height: Val::Percent(50.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            p.spawn(TextBundle {
                text: Text::from_section(
                    concat!("Gyra v", env!("CARGO_PKG_VERSION")),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),

                ..default()
            });

            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "Pos",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::GREEN_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            ..default()
                        },
                    ),
                ]),

                ..default()
            })
            .insert(PositionText);

            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "FPS",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::RED_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            ..default()
                        },
                    ),
                ]),

                ..default()
            })
            .insert(FpsText);
            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "Render",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::GREEN_100),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::RED_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::YELLOW_500),
                            ..default()
                        },
                    ),
                ]),

                ..default()
            })
            .insert(RenderText);
        })
        .insert(Menu);

    commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            style: Style {
                max_width: Val::Percent(30.0),
                max_height: Val::Percent(50.0),
                position_type: PositionType::Absolute,
                right: Val::Percent(0.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "Chunks",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::PINK_100),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::PURPLE_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::BLUE_500),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::RED_200),
                            ..default()
                        },
                    ),
                ]),

                ..default()
            })
            .insert(ChunkText);

            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "RAM",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::PINK_100),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::GREEN_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::GREEN_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::BLUE_200),
                            ..default()
                        },
                    ),
                ]),

                ..default()
            })
            .insert(MemoryUsageText);

            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "GPU",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::BLUE_100),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        format!(" {:?} ", rd.backend),
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::PURPLE_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        rd.name.clone(),
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::GREEN_200),
                            ..default()
                        },
                    ),
                ]),

                ..default()
            });

            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "CPU",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::PINK_100),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::GREEN_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::PURPLE_100),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::CYAN_100),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::RED_100),
                            ..default()
                        },
                    ),
                ]),
                ..default()
            })
            .insert(CpuText);
        })
        .insert(Menu);

    commands.insert_resource(DebugScreenActive);
}

fn debug_screen_handler(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    is_active: Option<Res<DebugScreenActive>>,
    mut menus: Query<&mut Visibility, With<Menu>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    if keys.just_pressed(KeyCode::F3) {
        for mut menu in menus.iter_mut() {
            if is_active.is_some() {
                *menu = Visibility::Hidden;
                commands.remove_resource::<DebugScreenActive>();
            } else {
                *menu = Visibility::Visible;
                commands.insert_resource(DebugScreenActive);
            }
        }
    }

    if keys.just_pressed(KeyCode::F4) {
        info!("Toggling wireframe");
        wireframe_config.global = !wireframe_config.global;
    }
}

fn update_diagnostics_text(
    mut cpu_q: Query<&mut Text, With<CpuText>>,
    mut mem_q: Query<&mut Text, (With<MemoryUsageText>, Without<CpuText>)>,
    mut events: EventReader<DiagnosticReport>,
) {
    let mut cpu = cpu_q.single_mut();
    let mut mem = mem_q.single_mut();

    for event in events.read() {
        let percent = (100.0 * event.memory_usage as f32) / event.avaliable_memory as f32;

        cpu.sections[1].value = format!(" C: {:.2}%", event.compute);
        cpu.sections[2].value = format!(" AC: {:.2}%", event.async_compute);
        cpu.sections[3].value = format!(" IO: {:.2}%", event.io);
        cpu.sections[4].value = format!(" M: {:.2}%", event.main);

        mem.sections[1].value = format!(" {:.2}%", percent);
        mem.sections[2].value = format!(" used: {:.2} MB", event.memory_usage / 1024 / 1024);
        mem.sections[3].value = format!(" free: {:.2} MB", event.avaliable_memory / 1024 / 1024);
    }
}

#[cfg(unix)]
fn update_diagnostics_values(
    mut cpu_writer: EventWriter<DiagnosticReport>,
    mut timer: ResMut<DiagnosticsTimer>,
    time: Res<Time>,
    mut system: Local<Option<sysinfo::System>>,
) {
    timer.timer.tick(time.delta());

    if system.is_none() {
        system.replace(sysinfo::System::new_all());
    }

    let system = system.as_mut().unwrap();
    let myself = Pid::from(std::process::id() as usize);

    if timer.timer.finished() {
        let avaliable_memory = system.available_memory();

        let mut children = vec![];

        for (nm, proc) in system.processes() {
            if let Some(parent) = proc.parent() {
                if parent == myself {
                    children.push(nm);
                }
            }
        }

        let myself_proc = system.process(myself).unwrap();
        let memory_usage = myself_proc.memory();

        let mut compute = vec![];
        let mut async_compute = vec![];
        let mut io = vec![];
        let mut main = vec![];

        for pid in children {
            let proc = system.process(*pid).unwrap();
            let name = proc.name().to_string_lossy();
            if name.starts_with("Compute") {
                bevy::log::info!("Compute CPU Usage: {:?}", proc.cpu_usage());
                compute.push(proc.cpu_usage());
            } else if name.contains("Async") {
                async_compute.push(proc.cpu_usage());
            } else if name.contains("IO") {
                io.push(proc.cpu_usage());
            } else if name.contains("main") {
                main.push(proc.cpu_usage());
            }
        }

        cpu_writer.send(DiagnosticReport {
            compute: compute.iter().sum::<f32>() / compute.len() as f32,
            async_compute: async_compute.iter().sum::<f32>() / async_compute.len() as f32,
            io: io.iter().sum::<f32>() / io.len() as f32,
            main: main.iter().sum::<f32>() / main.len() as f32,
            memory_usage,
            avaliable_memory,
        });

        system.refresh_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                .with_memory(MemoryRefreshKind::new().with_ram())
                .with_processes(ProcessRefreshKind::everything()),
        );
    }
}

#[cfg(windows)]
fn update_diagnostics_values(
    mut cpu_writer: EventWriter<DiagnosticReport>,
    mut timer: ResMut<DiagnosticsTimer>,
    time: Res<Time>,
    mut system: Local<Option<sysinfo::System>>,
) {
    timer.timer.tick(time.delta());

    if system.is_none() {
        system.replace(sysinfo::System::new());
    }

    let system = system.as_mut().unwrap();

    if timer.timer.finished() {
        let avaliable_memory = system.available_memory();

        let myself = Pid::from(std::process::id() as usize);

        let Some(myself_proc) = system.process(myself) else {
            // Are you sure that you don't know me? I'm the main process! Try again kiddo.
            system.refresh_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                    .with_memory(MemoryRefreshKind::new().with_ram())
                    .with_processes(ProcessRefreshKind::everything()),
            );
            return;
        };

        let memory_usage = myself_proc.memory();

        /* Why are you felling so insecure baby?
         * I'm just trying to get some information about you.
         * ain't from CIA. Prob.
         */
        let Ok(Some((compute, async_compute, io, main))) =
            (unsafe { collect_windows_thread_usage() })
        else {
            return;
        };

        cpu_writer.send(DiagnosticReport {
            compute,
            async_compute,
            io,
            main,
            memory_usage,
            avaliable_memory,
        });

        system.refresh_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                .with_memory(MemoryRefreshKind::new().with_ram())
                .with_processes(ProcessRefreshKind::everything()),
        );
    }
}

#[cfg(windows)]
type TimeReport = (
    SystemTime,
    Vec<(
        windows::Win32::Foundation::HANDLE,
        (
            windows::Win32::Foundation::FILETIME,
            windows::Win32::Foundation::FILETIME,
            String,
        ),
    )>,
);

#[cfg(windows)]
unsafe fn collect_windows_thread_usage() -> windows::core::Result<Option<(f32, f32, f32, f32)>> {
    use std::mem::size_of;
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };
    use windows::Win32::System::Threading::{GetThreadDescription, GetThreadTimes, OpenThread};

    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)?;

    let mut entry = THREADENTRY32 {
        dwSize: size_of::<THREADENTRY32>() as u32,
        ..Default::default()
    };

    Thread32First(snapshot, &mut entry)?;

    let mut cpu_usage = vec![];
    let mut async_cpu_usage = vec![];
    let mut io_usage = vec![];
    let mut main_cpu = vec![];

    let should_process = __WIN_LAST_TIME.with(|last_time| -> windows::core::Result<bool> {
        if last_time.borrow().is_none() {
            let mut first_times = vec![];

            loop {
                if entry.th32OwnerProcessID == id() {
                    // Hey, how are you?
                    let h_thread = OpenThread(
                        windows::Win32::System::Threading::THREAD_QUERY_INFORMATION,
                        false,
                        entry.th32ThreadID,
                    )?;

                    // What is your name?
                    let name = GetThreadDescription(h_thread)?;

                    // No.. No.. What is your *real* name? MARCOOOO
                    let real_name = name.to_string()?;

                    let mut creation_time = FILETIME::default();
                    let mut exit_time = FILETIME::default();
                    let mut kernel_time = FILETIME::default();
                    let mut user_time = FILETIME::default();

                    /* Now that we bought a Rolex we should use it to get time! yay :D */
                    GetThreadTimes(
                        h_thread,
                        &mut creation_time,
                        &mut exit_time,
                        &mut kernel_time,
                        &mut user_time,
                    )?;

                    first_times.push((h_thread, (kernel_time, user_time, real_name)));
                }

                if Thread32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }

            *last_time.borrow_mut() = Some((SystemTime::now(), first_times));
            return Ok(false);
        }

        Ok(true)
    })?;

    if !should_process {
        return Ok(None);
    }

    let (start_time, first_times) = __WIN_LAST_TIME.take().unwrap();

    let elapsed = start_time.elapsed().unwrap().as_secs_f32();

    for (h_thread, (prev_kernel, prev_user, real_name)) in first_times {
        let (kernel_time, user_time) = get_thread_times(h_thread)?;
        CloseHandle(h_thread)?;

        let kernel_diff = duration_to_f32(kernel_time) - duration_to_f32(prev_kernel);
        let user_diff = duration_to_f32(user_time) - duration_to_f32(prev_user);
        let total_time = kernel_diff + user_diff;
        let usage_percent = (total_time / elapsed) * 100.0;

        if real_name.starts_with("Compute") {
            cpu_usage.push(usage_percent);
        } else if real_name.contains("Async") {
            async_cpu_usage.push(usage_percent);
        } else if real_name.contains("IO") {
            io_usage.push(usage_percent);
        } else if real_name.contains("main") {
            main_cpu.push(usage_percent);
        }
    }

    // Thanks the small D guys that lower the average D size.
    fn average(cpu_usage: &[f32]) -> f32 {
        if cpu_usage.is_empty() {
            return 0.0;
        }

        cpu_usage.iter().sum::<f32>() / cpu_usage.len() as f32
    }

    Ok(Some((
        average(&cpu_usage),
        average(&async_cpu_usage),
        average(&io_usage),
        average(&main_cpu),
    )))
}

#[cfg(windows)]
fn duration_to_f32(time: windows::Win32::Foundation::FILETIME) -> f32 {
    const HUNDRED_NANOSECONDS: f32 = 10_000_000.0; // 1 second = 10 million 100-nanoseconds.
    let timestamp = ((time.dwHighDateTime as u64) << 32) | time.dwLowDateTime as u64;
    timestamp as f32 / HUNDRED_NANOSECONDS
}

#[cfg(windows)]
fn get_thread_times(
    h_thread: windows::Win32::Foundation::HANDLE,
) -> windows::core::Result<(
    windows::Win32::Foundation::FILETIME,
    windows::Win32::Foundation::FILETIME,
)> {
    use windows::Win32::{Foundation::FILETIME, System::Threading::GetThreadTimes};

    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();

    unsafe {
        GetThreadTimes(
            h_thread,
            &mut creation_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        )?;
    }

    Ok((kernel_time, user_time))
}
