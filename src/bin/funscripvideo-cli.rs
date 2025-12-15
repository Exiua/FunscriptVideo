use std::{path::PathBuf, process::ExitCode, result};

use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use tracing::{error, info, level_filters::LevelFilter, warn};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use FunScriptVideo::{db_client::DbClient, fsv::{self, AddArgs, EntryType, ItemType}};

#[derive(Parser, Debug)]
#[command(version = "v1.0.0", about = "FunscriptVideo CLI Utility", long_about = None, group(
    clap::ArgGroup::new("logging")
        .args(&["verbosity", "quiet", "silent"])
        .multiple(false)
        .required(false)
))]
struct Args {
    #[arg(short, long, global = true, default_value = "stdout", help = "Logging mode: none, stdout, file, both")]
    log_mode: LogMode,
    #[arg(
        short = 'v',
        long = "verbose",
        global = true,
        action = ArgAction::Count,
        help = "Increase verbosity: -v = debug, -vv = trace"
    )]
    verbosity: u8,
    #[arg(
        short = 'q',
        long = "quiet",
        global = true,
        action = ArgAction::Count,
        help = "Decrease verbosity: -q = warn, -qq = error"
    )]
    quiet: u8,
    #[arg(
        long,
        default_value_t = false,
        global = true,
        help = "Disable all logging output"
    )]
    silent: bool,
    /// Run in non-interactive mode (disable all user prompts)
    #[arg(long, global = true, help = "Disable interactive prompts (for scripting or CI)")]
    non_interactive: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validate a FunscriptVideo file
    Validate {
        #[arg(help = "Path to the FunscriptVideo file to validate")]
        path: PathBuf,
    },
    /// Create a new FunscriptVideo file
    Create {
        #[arg(help = "Path to the new FunscriptVideo file")]
        path: PathBuf,
        #[arg(help = "Title of the FunscriptVideo")]
        title: String,
        #[arg(num_args = 0.., help = "Tags associated with the FunscriptVideo")]
        tags: Vec<String>,
        #[arg(long, help = "Optional video file to include")]
        video: Option<PathBuf>,
        #[arg(long, help = "Optional video creator key")]
        video_creator_key: Option<String>,
        #[arg(long, help = "Optional script file to include")]
        script: Option<PathBuf>,
        #[arg(long, help = "Optional script creator key")]
        script_creator_key: Option<String>,
    },
    /// Add an entry to a FunscriptVideo file
    #[command(subcommand)]
    Add(AddCommands),
    /// Remove an entry from a FunscriptVideo file
    Remove {
        #[arg(help = "Path to the FunscriptVideo file to modify")]
        path: PathBuf,
        #[arg(help = "Type of entry to remove")]
        entry_type: EntryType,
        #[arg(help = "Identifier of the entry to remove (key for creator_info, filename for video/script/subtitle)")]
        entry_id: String,
        // TODO: Figure out how to cleanly add this option to the cli
        // #[arg()]
        // db: bool,
    },
    /// Extract contents from a FunscriptVideo file
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
    /// Display information about a FunscriptVideo file
    Info {
        #[arg(help = "Path to the FunscriptVideo file to display info for")]
        path: PathBuf,
    },
    /// Rebuild a FunscriptVideo file
    Rebuild {
        #[arg(help = "Path to the FunscriptVideo file to rebuild")]
        path: PathBuf,
    }
}

#[derive(Subcommand, Debug)]
enum AddCommands {
    /// Add a creator_info record to the database or FSV, depending on arguments
    #[command(subcommand)]
    Creator(CreatorLocation),
    /// Add a video file (with optional creator info) to an existing FSV container
    Video {
        #[arg(help = "Path to the FSV file to modify")]
        fsv_path: PathBuf,
        #[arg(help = "Path to the video file to add")]
        video_path: PathBuf,
        #[arg(long, help = "Optional creator key (must exist in DB)")]
        creator_key: Option<String>,
    },
    /// Add a script file (with optional creator info) to an existing FSV container
    Script {
        #[arg(help = "Path to the FSV file to modify")]
        fsv_path: PathBuf,
        #[arg(help = "Path to the script file to add")]
        script_path: PathBuf,
        #[arg(long, help = "Optional creator key (must exist in DB)")]
        creator_key: Option<String>,
    },
    /// Add a subtitle file (with optional creator info) to an existing FSV container
    Subtitle {
        #[arg(help = "Path to the FSV file to modify")]
        fsv_path: PathBuf,
        #[arg(help = "Path to the subtitle file to add")]
        subtitle_path: PathBuf,
        #[arg(long, help = "Optional creator key (must exist in DB)")]
        creator_key: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum CreatorLocation {
    Database {
        #[arg(help = "Name of the creator")]
        name: String,
        #[arg(required = true, help = "Unique creator key/identifier")]
        key: String,
        #[arg(num_args = 0.., help = "List of social URLs (e.g. --socials twitter.com/foo patreon.com/foo)")]
        socials: Vec<String>,
    },
    Fsv {
        #[arg(help = "Path to the FSV file to modify")]
        fsv_path: PathBuf,
        #[arg(help = "Type of work to associate the creator with")]
        work_type: ItemType,
        #[arg(short, long, required = true, help = "Creator key/identifier")]
        creator_key: String,
        #[arg(short, long, required = true, help = "Name of the created work")]
        work_name: String,
        #[arg(short, long, default_value = "", help = "Source URL")]
        source_url: String,
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogMode {
    None,
    Stdout,
    File,
    Both,
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

fn verbosity_to_level(count: u8) -> LogLevel {
    match count {
        0 => LogLevel::Info,
        1 => LogLevel::Debug,
        _ => LogLevel::Trace,
    }
}

fn quiet_to_level(count: u8) -> LogLevel {
    match count {
        0 => LogLevel::Info,
        1 => LogLevel::Warn,
        _ => LogLevel::Error,
    }
}


fn configure_logging(app_name: &str, mode: LogMode, level: LogLevel) -> WorkerGuard {
    let file_appender = rolling::daily("logs", format!("{}.log", app_name));
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let level_filter: LevelFilter = level.into();
    let env_filter = EnvFilter::builder()
        .with_default_directive(level_filter.into())
        .from_env_lossy();

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

fn main() -> ExitCode {
    let args = Args::parse();
    let level = if args.silent {
        LogLevel::Off
    }
    else if args.verbosity > 0 {
        verbosity_to_level(args.verbosity)
    }
    else if args.quiet > 0 {
        quiet_to_level(args.quiet)
    }
    else {
        LogLevel::Info
    };

    let _guard = configure_logging("funscripvideo-cli", args.log_mode, level);
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    if result.is_err() {
        error!("Failed to create Tokio runtime: {}", result.err().unwrap());
        return ExitCode::FAILURE;
    }

    let executable_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    if executable_dir.is_none() {
        error!("Failed to determine executable directory.");
        return ExitCode::FAILURE;
    }

    let executable_dir = executable_dir.unwrap();
    let database_path = executable_dir.join("funscripvideo.db");
    let rt = result.unwrap();
    let result = rt.block_on(DbClient::new(&database_path));
    if result.is_err() {
        error!("Failed to initialize database client: {}", result.err().unwrap());
        return ExitCode::FAILURE;
    }

    let db_client = result.unwrap();
    let interactive = !args.non_interactive;
    match args.command {
        Commands::Validate { path } => validate(&path),
        Commands::Create { path, title, tags, video, script, video_creator_key, script_creator_key } => rt.block_on(create(path, title, tags, video, script, video_creator_key, script_creator_key, &db_client, interactive)),
        Commands::Add(add_cmd) => rt.block_on(add(add_cmd, &db_client, interactive)),
        Commands::Remove { path, entry_type, entry_id } => remove(&path, entry_type, entry_id),
        Commands::Extract { path, output_dir } => extract(&path, &output_dir),
        Commands::Info { path } => info(&path),
        Commands::Rebuild { path } => rebuild(path),
    }

    ExitCode::SUCCESS
}

fn validate(path: &PathBuf) {
    let result = FunScriptVideo::fsv::validate_fsv(&path);
    match result {
        Ok(state) => match state {
            FunScriptVideo::fsv::FsvState::Valid => {
                info!("FSV file is valid.");
            }
            FunScriptVideo::fsv::FsvState::ContentIncomplete(reason) => match reason {
                FunScriptVideo::fsv::ContentIncompleteReason::UnableToReadItem(item_type) => warn!("Unable to read {} file", item_type.get_name_lower()),
                FunScriptVideo::fsv::ContentIncompleteReason::MissingItemFile(item_type) => warn!("Missing {} file in archive", item_type.get_name_lower()),
                FunScriptVideo::fsv::ContentIncompleteReason::ItemPasswordProtected(item_type) => warn!("{} file is password protected", item_type.get_name()),
                FunScriptVideo::fsv::ContentIncompleteReason::DuplicateItemEntry(item_type) => warn!("Duplicate {} entry in metadata", item_type.get_name_lower()),
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

async fn create(path: PathBuf, title: String, tags: Vec<String>, video: Option<PathBuf>, script: Option<PathBuf>, video_creator_key: Option<String>, script_creator_key: Option<String>, db_client: &DbClient, interactive: bool) {
    let args = FunScriptVideo::fsv::CreateArgs::new(path, title, tags, video, script, video_creator_key, script_creator_key);
    let result = FunScriptVideo::fsv::create_fsv(args, db_client, interactive).await;
    match result {
        Ok(_) => info!("FSV file created successfully."),
        Err(err) => error!("Error creating FSV file: {}", err),
    }
}

async fn add(cmd: AddCommands, db_client: &DbClient, interactive: bool) {
    match cmd {
        AddCommands::Creator(creator_location) => {
            match creator_location {
                CreatorLocation::Database { name, key, socials } => {
                    let creator_info = FunScriptVideo::metadata::CreatorInfo::new(name, socials);
                    let result = db_client.insert_creator_info(&key, &creator_info).await;
                    match result {
                        Ok(_) => info!("Creator info added to database successfully."),
                        Err(err) => error!("Error adding creator info to database: {}", err),
                    }
                },
                CreatorLocation::Fsv { fsv_path, work_type, creator_key, work_name, source_url } => {
                    let result = FunScriptVideo::fsv::add_creator_to_fsv(&fsv_path, work_type, &creator_key, &work_name, &source_url, db_client).await;
                    match result {
                        Ok(_) => info!("Creator info added to FSV file successfully."),
                        Err(err) => error!("Error adding creator info to FSV file: {}", err),
                    }
                },
            }
        },
        AddCommands::Video { fsv_path, video_path, creator_key } => add_item_to_fsv(fsv_path, ItemType::Video, video_path, creator_key, db_client, interactive).await,
        AddCommands::Script { fsv_path, script_path, creator_key } => add_item_to_fsv(fsv_path, ItemType::Script, script_path, creator_key, db_client, interactive).await,
        AddCommands::Subtitle { fsv_path, subtitle_path, creator_key } => add_item_to_fsv(fsv_path, ItemType::Subtitle, subtitle_path, creator_key, db_client, interactive).await,
    }
}

async fn add_item_to_fsv(fsv_path: PathBuf, item_type: ItemType, item_path: PathBuf, creator_key: Option<String>, db_client: &DbClient, interactive: bool) {
    let args = AddArgs::new(fsv_path, item_type, item_path, creator_key);
    let result = FunScriptVideo::fsv::add_to_fsv(args, db_client, interactive).await;
    match result {
        Ok(_) => info!("{} added to FSV file successfully.", item_type.get_name()),
        Err(err) => error!("Error adding {} to FSV file: {}", item_type.get_name(), err),
    }
}

fn remove(path: &PathBuf, entry_type: EntryType, entry_id: String) {
    let result = FunScriptVideo::fsv::remove_from_fsv(&path, entry_type, &entry_id);
    match result {
        Ok(_) => info!("Entry removed from FSV file successfully."),
        Err(err) => error!("Error removing entry from FSV file: {}", err),
    }
}

fn extract(path: &PathBuf, output_dir: &PathBuf) {
    let result = FunScriptVideo::fsv::extract_fsv(&path, &output_dir, false);
    match result {
        Ok(_) => info!("FSV file extracted successfully."),
        Err(err) => error!("Error extracting FSV file: {}", err),
    }
}

fn info(path: &PathBuf) {
    let result = FunScriptVideo::fsv::get_fsv_info(&path);
    let fsv_info = match result {
        Ok(info) => info,
        Err(err) => {
            error!("Error getting FSV file info: {}", err);
            return;
        }
    };

    println!("FSV File Info:");
    println!("Title: {}", fsv_info.title);
    let mut missing_video_file = false;
    if !fsv_info.videos.is_empty() {
        println!("Videos ({}):", fsv_info.videos.len());
        for (video_name, is_present) in &fsv_info.videos {
            println!("  {}: {}", video_name, if *is_present { "Present" } else { "Missing" });
            if !*is_present {
                missing_video_file = true;
            }
        }
    }

    let mut missing_script_file = false;
    if !fsv_info.scripts.is_empty() {
        println!("Scripts ({}):", fsv_info.scripts.len());
        for (script_name, is_present) in &fsv_info.scripts {
            println!("  {}: {}", script_name, if *is_present { "Present" } else { "Missing" });
            if !*is_present {
                missing_script_file = true;
            }
        }
    }

    let mut missing_subtitle_file = false;
    if !fsv_info.subtitles.is_empty() {
        println!("Subtitles ({}):", fsv_info.subtitles.len());
        for (subtitle_name, is_present) in &fsv_info.subtitles {
            println!("  {}: {}", subtitle_name, if *is_present { "Present" } else { "Missing" });
            if !*is_present {
                missing_subtitle_file = true;
            }
        }
    }

    if !fsv_info.extra_files.is_empty() {
        println!("WARNING: Extra files found in FSV archive ({}):", fsv_info.extra_files.len());
        for extra_file in &fsv_info.extra_files {
            println!("  {}", extra_file);
        }
    }

    if missing_video_file {
        println!("WARNING: Some video files are missing from the FSV archive.");
    }

    if missing_script_file {
        println!("WARNING: Some script files are missing from the FSV archive.");
    }

    if missing_subtitle_file {
        println!("WARNING: Some subtitle files are missing from the FSV archive.");
    }

    if fsv_info.videos.is_empty() || fsv_info.scripts.is_empty() {
        println!("Container State: Invalid (missing video or script)");
    }
    else if missing_video_file || missing_script_file {
        println!("Container State: Content Incomplete");
    }
    else {
        println!("Container State: Content Complete");
    }
}

fn rebuild(path: PathBuf) {
    let result = FunScriptVideo::fsv::rebuild_fsv(&path);
    match result {
        Ok(_) => info!("FSV file rebuilt successfully."),
        Err(err) => error!("Error rebuilding FSV file: {}", err),
    }
}