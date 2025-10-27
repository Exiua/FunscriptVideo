use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use tracing::{error, info, warn};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version = "v1.0.0", about = "FunscriptVideo CLI Utility", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "stdout", help = "Logging mode: none, stdout, file, both")]
    log_mode: LogMode,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Validate {
        #[arg(help = "Path to the FunscriptVideo file to validate")]
        path: PathBuf,
    },
    Create {},
    Add {},
    Remove {},
    Extract {
        #[arg(help = "Path to the FunscriptVideo file to extract from")]
        path: PathBuf,
        #[arg(
            short,
            long,
            default_value = ".",
            help = "Destination directory for extracted files. The extractor will create a new subdirectory named after the FSV file stem (e.g., 'foo.fsv' -> '<output_dir>/foo/')."
        )]
        output_dir: PathBuf,
    },
    Info {},
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogMode {
    None,
    Stdout,
    File,
    Both,
}

fn configure_logging(app_name: &str, mode: LogMode) -> WorkerGuard {
    let file_appender = rolling::daily("logs", format!("{}.log", app_name));
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter: EnvFilter;
    #[cfg(debug_assertions)]
    {
        env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));
    }
    #[cfg(not(debug_assertions))]
    {
        env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    }

    match mode {
        LogMode::None => {}
        LogMode::Stdout => {
            let stdout_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(false);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(stdout_layer)
                .init();
        }
        LogMode::File => {
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false) // no color codes in log file
                .with_target(false);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(file_layer)
                .init();
        }
        LogMode::Both => {
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false) // no color codes in log file
                .with_target(false);

            let stdout_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(false);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(file_layer)
                .with(stdout_layer)
                .init();
        }
    }

    _guard
}

fn main() {
    let args = Args::parse();
    let _guard = configure_logging("funscripvideo-cli", args.log_mode);
    match args.command {
        Commands::Validate { path } => validate(&path),
        Commands::Create {} => create(),
        Commands::Add {} => add(),
        Commands::Remove {} => remove(),
        Commands::Extract { path, output_dir } => extract(&path, &output_dir),
        Commands::Info {} => info(),
    }
}

fn validate(path: &PathBuf) {
    let result = FunScriptVideo::fsv::validate_fsv(&path);
    match result {
        Ok(state) => match state {
            FunScriptVideo::fsv::FsvState::Valid => {
                info!("FSV file is valid.");
            }
            FunScriptVideo::fsv::FsvState::ContentIncomplete(reason) => match reason {
                FunScriptVideo::fsv::ContentIncompleteReason::UnableToReadVideo => {
                    warn!("Unable to read video file.");
                }
                FunScriptVideo::fsv::ContentIncompleteReason::MissingVideoFile => {
                    warn!("Missing video file.");
                }
                FunScriptVideo::fsv::ContentIncompleteReason::VideoPasswordProtected => {
                    warn!("Video file is password protected.");
                }
                FunScriptVideo::fsv::ContentIncompleteReason::DuplicateVideoFormatEntry => {
                    error!("Duplicate video format entry found.");
                }
                FunScriptVideo::fsv::ContentIncompleteReason::UnableToReadScript => {
                    warn!("Unable to read script file.");
                }
                FunScriptVideo::fsv::ContentIncompleteReason::MissingScriptFile => {
                    warn!("Missing script file.");
                }
                FunScriptVideo::fsv::ContentIncompleteReason::ScriptPasswordProtected => {
                    warn!("Script file is password protected.");
                }
                FunScriptVideo::fsv::ContentIncompleteReason::DuplicateScriptVariantEntry => {
                    error!("Duplicate script variant entry found.");
                }
            },
            FunScriptVideo::fsv::FsvState::MetadataInvalid(reason) => match reason {
                FunScriptVideo::fsv::MetadataInvalidReason::InvalidFormatVersion => {
                    error!("Invalid format version in metadata.");
                }
                FunScriptVideo::fsv::MetadataInvalidReason::MalformedJson(json) => {
                    error!("Malformed JSON in metadata: {}", json);
                }
                FunScriptVideo::fsv::MetadataInvalidReason::UnsupportedFormatVersion(version) => {
                    error!("Unsupported format version in metadata: {}", version);
                }
                FunScriptVideo::fsv::MetadataInvalidReason::MissingVideoFormat => {
                    error!("Missing video format in metadata.");
                }
                FunScriptVideo::fsv::MetadataInvalidReason::MissingScriptVariant => {
                    error!("Missing script variant in metadata.");
                }
            },
        },
        Err(err) => {
            error!("Error validating FSV file: {}", err);
        }
    }
}

fn create() {
    todo!()
}

fn add() {
    todo!()
}

fn remove() {
    todo!()
}

fn extract(path: &PathBuf, output_dir: &PathBuf) {
    let result = FunScriptVideo::fsv::extract_fsv(&path, &output_dir, false);
    match result {
        Ok(_) => info!("FSV file extracted successfully."),
        Err(err) => error!("Error extracting FSV file: {}", err),
    }
}

fn info() {
    todo!()
}
