use crate::constants::{CONFIG_PATH, HISTORY};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Rere arguments.
#[derive(Parser, Debug)]
#[clap(name = "rere", version = "0.0.1")]
pub struct Args {
    /// Path to config file.
    #[clap(value_name = "CONFIG", default_value = CONFIG_PATH)]
    pub config: PathBuf,

    #[clap(subcommand)]
    pub command: Command,
}

/// Rere subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Record shell command args.
    Record,

    /// Replay and verify shell commands against recorded snapshot.
    Replay,

    /// Initialize a new rere config.
    Init {
        /// Override test file location relative to config file directory [default: test.list].
        #[clap(long, value_name = "FILE")]
        test_file: Option<PathBuf>,

        /// Overwrite default snapshots location relative to config file directory [default:
        /// snapshots/].
        #[clap(long, value_name = "DIR")]
        snapshot_dir: Option<PathBuf>,

        /// Override default number of snapshots to keep metadata history for [default: 3].
        #[clap(long, value_name = "NUM", default_value_t = HISTORY, hide_default_value = true)]
        history: usize,

        /// Set overwrite default for record command [default: true].
        #[clap(long, value_parser)]
        overwrite: Option<bool>,

        /// Set fail-fast default for replay command [default: true].
        #[clap(long, action)]
        fail_fast: Option<bool>,
    },

    /// Clean up snapshots, all testing files, or reset config.
    Clean {
        /// Clean up all testing files, directories, and subdirectories.
        #[clap(long)]
        all: bool,

        /// Clean up all snapshot files.
        #[clap(long)]
        snapshots: bool,

        /// Reset config file to defaults (except for `test_file` and `snapshot_dir` values).
        #[clap(long)]
        config: bool,
    },
}
