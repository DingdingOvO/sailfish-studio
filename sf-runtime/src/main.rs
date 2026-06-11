mod commands;
mod config;
mod error;
mod project;

use clap::Parser;

/// Sailfish Studio Runtime CLI
#[derive(Parser, Debug)]
#[command(name = "sf", version, about = "Sailfish Studio Runtime CLI")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

/// Available CLI commands.
#[derive(clap::Subcommand, Debug)]
enum CliCommand {
    /// Run a .sfl or .sfp file
    Run {
        /// Path to the .sfl or .sfp file
        file: std::path::PathBuf,

        /// Run in headed mode (with graphical window)
        #[arg(long)]
        headed: bool,

        /// Target FPS (frames per second)
        #[arg(long)]
        fps: Option<u32>,

        /// Stage width in pixels
        #[arg(long)]
        width: Option<u32>,

        /// Stage height in pixels
        #[arg(long)]
        height: Option<u32>,
    },

    /// Package project as .sfp
    Pack {
        /// Path to the project file or directory
        file: std::path::PathBuf,

        /// Output path for the .sfp file
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Embed the runtime binary in the package
        #[arg(long)]
        embed_runtime: bool,
    },

    /// Create a new project
    New {
        /// Project name
        name: String,

        /// Project template (blank, game, animation)
        #[arg(short, long, default_value = "blank")]
        template: Option<String>,

        /// Directory to create the project in (defaults to current directory)
        #[arg(short, long)]
        dir: Option<std::path::PathBuf>,
    },

    /// Check project syntax
    Check {
        /// Path to the .sfl or .sfp file
        file: std::path::PathBuf,

        /// Enable strict mode (also check for warnings)
        #[arg(long)]
        strict: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = commands::dispatch(cli.command) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
