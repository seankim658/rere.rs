use crate::constants::{HISTORY, SNAPSHOT_DIR, TEST_FILE};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub common: CommonConfig,
    #[serde(default)]
    pub record: RecordConfig,
    #[serde(default)]
    pub replay: ReplayConfig,
    #[serde(default)]
    pub state: StateConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CommonConfig {
    pub test_file: PathBuf,
    pub snapshot_dir: PathBuf,
    pub history: usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RecordConfig {
    pub overwrite: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ReplayConfig {
    pub fail_fast: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StateConfig {
    pub latest_snapshots: Vec<PathBuf>,
    pub timestamps: Vec<DateTime<Utc>>,
    pub elapsed_time: Vec<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                test_file: PathBuf::from(TEST_FILE),
                snapshot_dir: PathBuf::from(SNAPSHOT_DIR),
                history: HISTORY,
            },
            record: RecordConfig { overwrite: true },
            replay: ReplayConfig { fail_fast: false },
            state: StateConfig::default(),
        }
    }
}

impl Config {
    pub fn load_or_create(config_path: &Path) -> Result<Self> {
        if config_path.exists() {
            let contents = fs::read_to_string(config_path).context("Failed to read config file")?;
            toml::from_str(&contents).context("Failed to parse config file")
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let config_str = toml::to_string_pretty(&self)?;
        fs::write(path, config_str).context("Failed to write config file")?;
        Ok(())
    }

    pub fn init_snapshot_dir(&self, base_dir: &Path, snapshot_dir: &Path) -> Result<()> {
        fs::create_dir_all(base_dir.join(snapshot_dir))?;
        Ok(())
    }

    pub fn update_latest_snapshot(
        &mut self,
        config_path: &PathBuf,
        snapshot_path: PathBuf,
        elapsed: Duration,
    ) -> Result<()> {
        let timestamp = Utc::now();

        self.state.latest_snapshots.insert(0, snapshot_path);
        self.state.timestamps.insert(0, timestamp);
        self.state.elapsed_time.insert(0, elapsed);

        // Trim to history limit.
        let limit = self.common.history;
        self.state.latest_snapshots.truncate(limit);
        self.state.timestamps.truncate(limit);
        self.state.elapsed_time.truncate(limit);

        self.save(config_path)
    }

    pub fn get_latest_snapshot(&self) -> Option<&PathBuf> {
        self.state.latest_snapshots.first()
    }
}

pub fn init_config(
    config_path: &Path,
    test_file: Option<PathBuf>,
    snapshot_dir: Option<PathBuf>,
    history: usize,
    overwrite: Option<bool>,
    fail_fast: Option<bool>,
) -> Result<Config> {
    if config_path.exists() {
        anyhow::bail!(
            "Configuration file already exists at {}",
            config_path.display()
        );
    }

    let mut config = Config::default();
    config.common.history = history;

    if let Some(test_file) = test_file {
        config.common.test_file = test_file;
    }
    if let Some(snapshot_dir) = snapshot_dir {
        config.common.snapshot_dir = snapshot_dir;
    }
    if let Some(overwrite) = overwrite {
        config.record.overwrite = overwrite;
    }
    if let Some(fail_fast) = fail_fast {
        config.replay.fail_fast = fail_fast;
    }

    config.init_snapshot_dir(config_path.parent().unwrap(), &config.common.snapshot_dir)?;
    config.save(config_path)?;
    Ok(config)
}
