use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser, Debug)]
#[command(version = "v1.0.0", about = "FunscriptVideo CLI Utility", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Validate {
        #[arg(help = "Path to the FunscriptVideo file to validate")]
        path: PathBuf,
    },
    Create {

    },
    Add {

    },
    Remove {

    },
    Extract {

    },
    Info {

    },
}

pub fn configure_logging(app_name: &str, stdout: bool) -> WorkerGuard {
    let file_appender = rolling::daily("logs", format!("{}.log", app_name));
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter: EnvFilter;
    #[cfg(debug_assertions)] {
    env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));
    } 
    #[cfg(not(debug_assertions))] {
    env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    }

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false) // no color codes in log file
        .with_target(false);

    if stdout {
        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_target(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    }
    else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    _guard
}

fn main() {
    let _guard = configure_logging("funscripvideo-cli", true);
    let args = Args::parse();

}