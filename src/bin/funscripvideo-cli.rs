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
    Info {

    },
}

#[derive(Debug)]
enum LogMode {
    None,
    Stdout,
    File,
    Both,
}

// pub fn configure_logging(app_name: &str, stdout: bool) -> WorkerGuard {
//     let file_appender = rolling::daily("logs", format!("{}.log", app_name));
//     let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

//     let env_filter: EnvFilter;
//     #[cfg(debug_assertions)] {
//     env_filter = EnvFilter::try_from_default_env()
//         .unwrap_or_else(|_| EnvFilter::new("debug"));
//     } 
//     #[cfg(not(debug_assertions))] {
//     env_filter = EnvFilter::try_from_default_env()
//         .unwrap_or_else(|_| EnvFilter::new("info"));
//     }

//     let file_layer = tracing_subscriber::fmt::layer()
//         .with_writer(non_blocking)
//         .with_ansi(false) // no color codes in log file
//         .with_target(false);

//     if stdout {
//         let stdout_layer = tracing_subscriber::fmt::layer()
//             .with_writer(std::io::stdout)
//             .with_target(false);

//         tracing_subscriber::registry()
//             .with(env_filter)
//             .with(file_layer)
//             .with(stdout_layer)
//             .init();
//     }
//     else {
//         tracing_subscriber::registry()
//             .with(env_filter)
//             .with(file_layer)
//             .init();
//     }

//     _guard
// }

fn configure_logging(app_name: &str, mode: LogMode) -> WorkerGuard {
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

    match mode {
        LogMode::None => {

        },
        LogMode::Stdout => {
            let stdout_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(false);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(stdout_layer)
                .init();
        },
        LogMode::File => {
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false) // no color codes in log file
                .with_target(false);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(file_layer)
                .init();
        },
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
        },
    }

    _guard
}

fn main() {
    let _guard = configure_logging("funscripvideo-cli", LogMode::Both);
    let args = Args::parse();
    match args.command {
        Commands::Validate { path } => todo!(),
        Commands::Create {  } => todo!(),
        Commands::Add {  } => todo!(),
        Commands::Remove {  } => todo!(),
        Commands::Extract { path, output_dir  } => todo!(),
        Commands::Info {  } => todo!(),
    }
}