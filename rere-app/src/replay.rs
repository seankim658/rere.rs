use crate::{
    config::{Config, ReplayResult},
    shell::capture,
};
use anyhow::Result;
use bi_parser::prelude::*;
use std::{fs::File, path::PathBuf, usize};

pub struct ReplayDiff {
    pub shell: String,
    pub field: String,
    pub expected: String,
    pub actual: String,
}

pub fn replay(config: &mut Config, config_path: &PathBuf) -> Result<()> {
    let test_file = config.common.test_file.clone();
    let base_dir = config_path.parent().unwrap();
    let test_path = base_dir.join(test_file);

    if !test_path.exists() {
        anyhow::bail!(
            "Test file not found at: {}. Run `rere init` first.",
            test_path.display()
        );
    }

    // Get latest snapshot
    let snapshot_filename = match config.get_latest_snapshot() {
        Some(path) => path,
        None => anyhow::bail!("No snapshots found. Run `rere record` first."),
    };
    let snapshot_path = base_dir
        .join(&config.common.snapshot_dir)
        .join(snapshot_filename);

    if !snapshot_path.exists() {
        anyhow::bail!(
            "Snapshot file not found at: {}. Run `rere record` first.",
            snapshot_path.display()
        );
    };

    // Read test list
    let shells = std::fs::read_to_string(test_path)?
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .map(|s| s.trim().to_owned())
        .collect::<Vec<_>>();

    // Read snapshot
    let mut reader = BiReader::new(File::open(snapshot_path)?);
    let count_field = reader.read_field_default()?;
    let count = match count_field {
        BiField::Integer { value, .. } => value as usize,
        _ => anyhow::bail!("Expected integer count field"),
    };

    if count != shells.len() {
        anyhow::bail!(
            "Number of commands in test file ({}) doesn't match snapshot ({})",
            shells.len(),
            count
        );
    }

    let start = std::time::Instant::now();
    let mut failed = false;

    // Replay each command and compare outputs
    for shell in shells {
        println!("Replaying: {}", shell);

        // Read expected output from snapshot
        let expected_shell = match reader.read_field_default()? {
            BiField::Blob { data, .. } => String::from_utf8(data)?,
            _ => anyhow::bail!("Expected blob field for shell command"),
        };

        if shell != expected_shell {
            print_diff("shell command", &expected_shell, &shell);
            if config.replay.fail_fast {
                failed = true;
                break;
            }
        }

        let expected_returncode = match reader.read_field_default()? {
            BiField::SignedInteger { value, .. } => value,
            _ => anyhow::bail!("Expected integer field for return code"),
        };

        let expected_stdout = match reader.read_field_default()? {
            BiField::Blob { data, .. } => data,
            _ => anyhow::bail!("Expected blob field for stdout"),
        };

        let expected_stderr = match reader.read_field_default()? {
            BiField::Blob { data, .. } => data,
            _ => anyhow::bail!("Expected blob field for stderr"),
        };

        // Capture actual output
        let output = capture(&shell)?;

        // Compare outputs
        if output.returncode as i64 != expected_returncode {
            print_diff(
                "return code",
                &expected_returncode.to_string(),
                &output.returncode.to_string(),
            );
            if config.replay.fail_fast {
                failed = true;
                break;
            }
        }

        if output.stdout != expected_stdout {
            print_diff(
                "stdout",
                &String::from_utf8_lossy(&expected_stdout),
                &String::from_utf8_lossy(&output.stdout),
            );
            if config.replay.fail_fast {
                failed = true;
                break;
            }
        }

        if output.stderr != expected_stderr {
            print_diff(
                "stderr",
                &String::from_utf8_lossy(&expected_stderr),
                &String::from_utf8_lossy(&output.stderr),
            );
            if config.replay.fail_fast {
                failed = true;
                break;
            }
        }
    }

    // Update config with replay results
    let elapsed = chrono::Duration::from_std(start.elapsed())?;
    let result = if failed {
        ReplayResult::Fail
    } else {
        ReplayResult::Pass
    };
    config.update_latest_replay(config_path, elapsed, result)?;

    if failed {
        anyhow::bail!("Replay failed");
    }

    println!("All tests passed!");
    Ok(())
}

fn print_diff(field: &str, expected: &str, actual: &str) {
    println!("\nUnexpected {field}:");
    println!("  Expected: {}", expected);
    println!("  Actual: {}", actual);
}
