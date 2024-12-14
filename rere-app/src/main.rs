mod cli;
mod config;
pub mod constants;
mod record;
mod replay;
mod shell;

use anyhow::Result;
use clap::Parser;
use cli::{Args, Command};
use config::{init_config, Config};
use std::fs;

fn main() -> Result<()> {
    let args = Args::parse();

    if !matches!(args.command, Command::Init { .. }) {
        let base_dir = args.config.parent().unwrap();
        if !base_dir.exists() {
            anyhow::bail!(
                "Testing environment not found at: {}. Run `rere init` first.",
                base_dir.display()
            );
        }
    }

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
            match record::record(&mut config, &args.config) {
                Ok(_) => println!("Recording completed successfully"),
                Err(e) => {
                    eprintln!("Error during recording: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Replay => {
            let mut config = Config::load_or_create(&args.config)?;
            match replay::replay(&mut config, &args.config) {
                Ok(_) => println!("Recording completed successfully"),
                Err(e) => {
                    eprintln!("Error during recording: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Clean {
            all,
            snapshots,
            config,
        } => {
            handle_clean(&args, all, snapshots, config)?;
        }
    }

    Ok(())
}

fn handle_clean(args: &Args, all: bool, snapshots: bool, config: bool) -> Result<()> {
    validate_clean_flags(all, snapshots, config)?;

    let mut cfg = Config::load_or_create(&args.config)?;
    let base_dir = args.config.parent().unwrap();

    if all {
        let message = format!(
            "This will delete ALL testing files, directories, and subdirectories:\n\
            - Directory: {}",
            base_dir.display()
        );

        if !prompt_confirmation(&message)? {
            println!("Operation cancelled");
            return Ok(());
        }

        if !base_dir.exists() {
            anyhow::bail!("Couldn't find directory at `{}`.", base_dir.display());
        }
        fs::remove_dir_all(base_dir)?;
        println!("Removed all testing files and directories");
    } else {
        let mut message = String::from("This will:");

        if snapshots {
            let snapshot_dir = base_dir.join(&cfg.common.snapshot_dir);
            message.push_str(&format!(
                "\n- Delete all snapshots in: {}",
                snapshot_dir.display()
            ));
        }

        if config {
            message.push_str(&format!(
                "\n- Reset config at `{}` to default values\
                (preserving test_file and snapshot_dir values)",
                args.config.display()
            ));
        }

        if !prompt_confirmation(&message)? {
            println!("Operation cancelled");
            return Ok(());
        }

        if snapshots {
            let snapshot_dir = base_dir.join(&cfg.common.snapshot_dir);
            if snapshot_dir.exists() {
                fs::remove_dir_all(&snapshot_dir)?;
                fs::create_dir(&snapshot_dir)?;
                println!("Cleared snapshots directory");
            }
        }

        if config {
            let test_file = cfg.common.test_file.clone();
            let snapshot_dir = cfg.common.snapshot_dir.clone();
            cfg = Config::default();
            cfg.common.test_file = test_file;
            cfg.common.snapshot_dir = snapshot_dir;
            cfg.save(&args.config)?;
            println!("Reset config file at `{}`", &args.config.display());
        }
    }

    Ok(())
}

fn validate_clean_flags(all: bool, snapshots: bool, config: bool) -> Result<()> {
    if all && (snapshots || config) {
        anyhow::bail!(
            "Invalid option combination: `--all` flag cannot be used with other clean flags"
        );
    }
    if !all && !snapshots && !config {
        anyhow::bail!("At least one cleanup option must be specified");
    }

    Ok(())
}

fn prompt_confirmation(message: &str) -> Result<bool> {
    println!("\n{}", message);
    println!("\nDo you want to continue? [y/n]: ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_lowercase() == "y")
}
