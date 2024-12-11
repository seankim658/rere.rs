mod cli;
mod config;
pub mod constants;
mod record;
mod shell;

use anyhow::Result;
use clap::Parser;
use cli::{Args, Command};
use config::{init_config, Config};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init {
            test_file,
            snapshot_dir,
            history,
            overwrite,
            fail_fast,
        } => {
            init_config(
                &args.config,
                test_file,
                snapshot_dir,
                history,
                overwrite,
                fail_fast,
            )?;
            println!("Initialized config at {}", args.config.display());
        }
        Command::Record => {
            let mut config = Config::load_or_create(&args.config)?;
            record::record(&mut config, &args.config)?;
            println!("Recording completed successfully");
        }
    }

    Ok(())
}
