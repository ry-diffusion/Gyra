use godot::global::{godot_error, godot_print_rich, godot_warn};
use log::SetLoggerError;
use log::{Level, LevelFilter, Metadata, Record};

pub struct RysterGodotLogger;
static LOGGER: RysterGodotLogger = RysterGodotLogger;

impl log::Log for RysterGodotLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            /* Set log level with bb_code */
            let level_pretty = match record.level() {
                Level::Error => "[color=#ff0000]ERROR[/color]",
                Level::Warn => "[color=#ffcc00]WARN[/color]",
                Level::Info => "[color=#00ff00]INFO[/color]",
                Level::Debug => "[color=#0000ff]DEBUG[/color]",
                Level::Trace => "[color=#ff00ff]TRACE[/color]",
            };

            match record.level() {
                Level::Error => godot_error!("{}", record.args()),
                Level::Warn => godot_warn!("{}", record.args()),
                _ => godot_print_rich!("{}: {}", level_pretty, record.args()),
            }
        }
    }

    fn flush(&self) {}
}

pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level))
}
