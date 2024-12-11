mod cli;
mod config;
pub mod constants;

use anyhow::Result;
use clap::Parser;
use cli::{Args, Command};
use config::{init_config, Config};
use std::path::Path;

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
            init_config(&args.config, test_file, snapshot_dir, history, overwrite, fail_fast)?;
            println!("Initialized config at {}", args.config.display());
        }
        Command::Record => {
            let config = Config::load_or_create(&args.config)?;
            // TODO
            //println!(
            //    "Recording from {}",
            //    input.unwrap_or(config.common.test_file.display())
            //);
        }
    }

    Ok(())
}
