use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xsnap")]
#[command(about = "Visual regression testing tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run visual regression tests
    Test {
        /// Path to config file
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,

        /// Disable automatic snapshot creation for new tests
        #[arg(long)]
        no_create: bool,

        /// Disable .only behavior
        #[arg(long)]
        no_only: bool,

        /// Disable .skip behavior
        #[arg(long)]
        no_skip: bool,

        /// Filter tests by name pattern
        #[arg(long)]
        filter: Option<String>,

        /// Run tests in pipeline mode
        #[arg(long)]
        pipeline: bool,

        /// Number of parallel browser instances
        #[arg(long)]
        parallelism: Option<usize>,
    },

    /// Approve failing snapshots as new baselines
    Approve {
        /// Path to config file
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,

        /// Approve all failing snapshots
        #[arg(long)]
        all: bool,

        /// Filter tests by name pattern
        #[arg(long)]
        filter: Option<String>,
    },

    /// Clean up orphaned snapshots
    Cleanup {
        /// Path to config file
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,
    },

    /// Migrate snapshots between directories
    Migrate {
        /// Source directory
        #[arg(long, default_value = ".")]
        source: String,

        /// Target directory
        #[arg(long, default_value = ".")]
        target: String,
    },

    /// Initialize a new xsnap configuration
    Init,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            config,
            no_create,
            no_only,
            no_skip,
            filter,
            pipeline,
            parallelism,
        } => {
            let _ = (config, no_create, no_only, no_skip, filter, pipeline, parallelism);
            todo!("Implement test command")
        }
        Commands::Approve {
            config,
            all,
            filter,
        } => {
            let _ = (config, all, filter);
            todo!("Implement approve command")
        }
        Commands::Cleanup { config } => {
            let _ = config;
            todo!("Implement cleanup command")
        }
        Commands::Migrate { source, target } => {
            let _ = (source, target);
            todo!("Implement migrate command")
        }
        Commands::Init => {
            todo!("Implement init command")
        }
    }
}
