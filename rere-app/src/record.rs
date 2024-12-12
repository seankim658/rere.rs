use crate::{
    config::Config,
    shell::{capture, CommandOutput},
};
use anyhow::Result;
use bi_parser::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

pub fn record(config: &mut Config, config_path: &PathBuf) -> Result<()> {
    let test_file = config.common.test_file.clone();
    let base_dir = config_path.parent().unwrap();
    let test_path = base_dir.join(test_file);

    // Read test list
    let shells = std::fs::read_to_string(test_path)?
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .map(|s| s.trim().to_owned())
        .collect::<Vec<_>>();

    // Capture outputs
    let start = std::time::Instant::now();
    let outputs: Result<Vec<CommandOutput>> = shells.iter().map(|shell| capture(shell)).collect();
    let outputs = outputs?;

    // Generate snapshot filename with timestamp
    let timestamp = chrono::Utc::now();
    let snapshot_name = match config.record.overwrite {
        true => {
            format!("{}.bi", config.common.test_file.display())
        }
        false => {
            format!(
                "{}_{}.bi",
                config.common.test_file.display(),
                timestamp.format("%Y%m%d_%H%M%S")
            )
        }
    };
    let snapshot_path = base_dir
        .join(&config.common.snapshot_dir)
        .join(snapshot_name);

    // Write snapshot using bi-parser library
    write_snapshot(&snapshot_path, &outputs)?;

    // Update config with new snapshot
    let elapsed = chrono::Duration::from_std(start.elapsed())?;
    config.update_latest_snapshot(config_path, snapshot_path, elapsed)?;

    Ok(())
}

fn write_snapshot(path: &Path, outputs: &[CommandOutput]) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BiWriter::new(file);

    // Write count of tests
    writer.write_field_default(&BiField::Integer {
        name: b"count".to_vec(),
        value: outputs.len() as u64,
    })?;

    // Write each test output
    for output in outputs {
        writer.write_field_default(&BiField::Blob {
            name: b"shell".to_vec(),
            data: output.shell.as_bytes().to_vec(),
        })?;
        writer.write_field_default(&BiField::SignedInteger {
            name: b"returncode".to_vec(),
            value: output.returncode as i64,
        })?;
        writer.write_field_default(&BiField::Blob {
            name: b"stdout".to_vec(),
            data: output.stdout.clone(),
        })?;
        writer.write_field_default(&BiField::Blob {
            name: b"stderr".to_vec(),
            data: output.stderr.clone(),
        })?;
    }

    Ok(())
}
