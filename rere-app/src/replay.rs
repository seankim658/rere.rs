use crate::{
    config::{Config, DiffContent, ReplayDiff, ReplayResult},
    record::load_test_commands,
    shell::capture,
};
use anyhow::Result;
use bi_parser::prelude::*;
use std::{fs::File, path::PathBuf, usize};

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
    let shells = load_test_commands(&test_path)?;

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
    let mut diffs = Vec::new();

    // Replay each command and compare outputs
    for shell in shells {
        println!("Replaying: {}", shell);

        // Read expected output from snapshot
        let expected_shell = match reader.read_field_default()? {
            BiField::Blob { data, .. } => String::from_utf8(data)?,
            _ => anyhow::bail!("Expected blob field for shell command"),
        };

        if shell != expected_shell {
            let diff = ReplayDiff {
                shell: shell.clone(),
                field: "shell command".to_owned(),
                expected: DiffContent::Single(expected_shell),
                actual: DiffContent::Single(shell.clone()),
            };
            print_diff(&diff);
            diffs.push(diff);
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
            let diff = ReplayDiff {
                shell: shell.clone(),
                field: "return code".to_owned(),
                expected: DiffContent::Single(expected_returncode.to_string()),
                actual: DiffContent::Single(output.returncode.to_string()),
            };
            print_diff(&diff);
            diffs.push(diff);
            if config.replay.fail_fast {
                failed = true;
                break;
            }
        }

        if output.stdout != expected_stdout {
            let diff = ReplayDiff {
                shell: shell.clone(),
                field: "stdout".to_owned(),
                expected: DiffContent::Lines(
                    String::from_utf8_lossy(&expected_stdout)
                        .lines()
                        .map(|s| s.to_string())
                        .collect(),
                ),
                actual: DiffContent::Lines(
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|s| s.to_string())
                        .collect(),
                ),
            };
            print_diff(&diff);
            diffs.push(diff);
            if config.replay.fail_fast {
                failed = true;
                break;
            }
        }

        if output.stderr != expected_stderr {
            let diff = ReplayDiff {
                shell: shell.clone(),
                field: "stderr".to_owned(),
                expected: DiffContent::Lines(
                    String::from_utf8_lossy(&expected_stderr)
                        .lines()
                        .map(|s| s.to_string())
                        .collect(),
                ),
                actual: DiffContent::Lines(
                    String::from_utf8_lossy(&output.stderr)
                        .lines()
                        .map(|s| s.to_string())
                        .collect(),
                ),
            };
            print_diff(&diff);
            diffs.push(diff);
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
    config.update_latest_replay(config_path, elapsed, result, diffs)?;

    if failed {
        anyhow::bail!("Replay failed");
    }

    println!("All tests passed!");
    Ok(())
}

fn print_diff(diff: &ReplayDiff) {
    println!("\nUnexpected {}:", diff.field);
    match (&diff.expected, &diff.actual) {
        (DiffContent::Single(expected), DiffContent::Single(actual)) => {
            println!("  Expected: {}", expected);
            println!("  Actual: {}", actual)
        }
        (DiffContent::Lines(expected), DiffContent::Lines(actual)) => {
            println!("  Expected:");
            for line in expected {
                println!("    {}", line);
            }
            println!("  Actual:");
            for line in actual {
                println!("    {}", line);
            }
        }
        _ => unreachable!("Mismatched diff content types"),
    }
}
