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
        /// Filter tests by name pattern
        #[arg(short, long)]
        filter: Option<String>,

        /// Update baseline snapshots instead of comparing
        #[arg(short, long)]
        update: bool,

        /// Path to config file
        #[arg(short, long)]
        config: Option<String>,

        /// Number of parallel browser instances
        #[arg(short, long)]
        parallelism: Option<usize>,

        /// Only run tests matching these tags
        #[arg(long)]
        tag: Vec<String>,

        /// Fail fast on first mismatch
        #[arg(long)]
        fail_fast: bool,
    },

    /// Approve failing snapshots as new baselines
    Approve {
        /// Approve all failing snapshots
        #[arg(short, long)]
        all: bool,

        /// Approve specific test by name
        #[arg(short, long)]
        test: Option<String>,

        /// Interactive approval mode
        #[arg(short, long)]
        interactive: bool,
    },

    /// Clean up orphaned snapshots
    Cleanup {
        /// Dry run - show what would be deleted
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Migrate snapshots after test changes
    Migrate {
        /// Old test name
        #[arg(long)]
        from: String,

        /// New test name
        #[arg(long)]
        to: String,
    },

    /// Initialize a new xsnap configuration
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            filter,
            update,
            config,
            parallelism,
            tag,
            fail_fast,
        } => {
            let _ = (filter, update, config, parallelism, tag, fail_fast);
            todo!("Implement test command")
        }
        Commands::Approve {
            all,
            test,
            interactive,
        } => {
            let _ = (all, test, interactive);
            todo!("Implement approve command")
        }
        Commands::Cleanup { dry_run } => {
            let _ = dry_run;
            todo!("Implement cleanup command")
        }
        Commands::Migrate { from, to } => {
            let _ = (from, to);
            todo!("Implement migrate command")
        }
        Commands::Init { force } => {
            let _ = force;
            todo!("Implement init command")
        }
    }
}
