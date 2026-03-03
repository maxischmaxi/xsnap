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
async fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Test {
            config,
            no_create,
            no_only,
            no_skip,
            filter,
            pipeline,
            parallelism,
        } => {
            match xsnap::commands::test::run_test(xsnap::commands::test::TestOptions {
                config,
                no_create,
                no_only,
                no_skip,
                filter,
                pipeline,
                parallelism,
            })
            .await
            {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("Error: {e}");
                    4
                }
            }
        }
        Commands::Approve {
            config,
            all,
            filter,
        } => {
            match xsnap::commands::approve::run_approve(
                xsnap::commands::approve::ApproveOptions {
                    config,
                    all,
                    filter,
                },
            ) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("Error: {e}");
                    2
                }
            }
        }
        Commands::Cleanup { config } => {
            match xsnap::commands::cleanup::run_cleanup(
                xsnap::commands::cleanup::CleanupOptions { config },
            ) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("Error: {e}");
                    2
                }
            }
        }
        Commands::Migrate { source, target } => {
            match xsnap::commands::migrate::run_migrate(
                xsnap::commands::migrate::MigrateOptions { source, target },
            ) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("Error: {e}");
                    2
                }
            }
        }
        Commands::Init => match xsnap::commands::init::run_init() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("Error: {e}");
                4
            }
        },
    };

    std::process::exit(exit_code);
}
