use flexi_logger::{Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, LoggerHandle, Naming, WriteMode};
use log::Record;
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

const LOG_FILES_TO_KEEP: usize = 3;

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();
static LOGGER: OnceLock<LoggerHandle> = OnceLock::new();

fn file_format(write: &mut dyn Write, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
    write!(
        write,
        "{} [{}] - {} - {}",
        now.now().format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        record.target(),
        record.args()
    )
}

fn console_format(write: &mut dyn Write, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
    let color_code = match record.level() {
        log::Level::Error => "\x1b[31m", // Red
        log::Level::Warn => "\x1b[33m",  // Yellow
        log::Level::Info => "\x1b[32m",  // Green
        log::Level::Debug => "\x1b[34m", // Blue
        log::Level::Trace => "\x1b[35m", // Magenta
    };

    write!(write, "{}", color_code)?;
    file_format(write, now, record)?;
    write!(write, "\x1b[0m")
}

pub fn init(log_dir: &Path) {
    let _ = LOG_DIR.set(log_dir.to_path_buf());

    let log_file_spec = FileSpec::default().directory(log_dir).basename("output").suffix("txt");

    let result = Logger::try_with_str("info").and_then(|logger| {
        logger
            .log_to_file(log_file_spec)
            .format_for_files(file_format)
            .format_for_stdout(console_format)
            .rotate(
                Criterion::Size(5_000_000),
                Naming::Timestamps,
                Cleanup::KeepLogFiles(LOG_FILES_TO_KEEP),
            )
            .write_mode(WriteMode::BufferAndFlush)
            .duplicate_to_stdout(Duplicate::All)
            .start()
    });

    match result {
        Ok(handle) => {
            let _ = LOGGER.set(handle);
        }
        Err(e) => eprintln!("Failed to initialize logger: {e}"),
    }
}

pub fn open_log_dir() {
    let Some(log_dir) = LOG_DIR.get() else {
        return;
    };

    if let Err(e) = Command::new("explorer").arg(log_dir).spawn() {
        log::error!("Failed to open log directory: {}", e);
    }
}

pub fn log(message: String, level: &str) {
    match level {
        "trace" => log::trace!("UI: {}", message),
        "debug" => log::debug!("UI: {}", message),
        "info" => log::info!("UI: {}", message),
        "warn" => log::warn!("UI: {}", message),
        "error" => log::error!("UI: {}", message),
        _ => log::info!("UI: {}", message),
    }
}
