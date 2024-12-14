use crate::constants::{FAILFAST, HISTORY, OVERWRITE, SNAPSHOT_DIR, TEST_FILE};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::{fs, u64};

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
    pub record_timestamps: Vec<DateTime<Utc>>,
    /// Vector of elapsed times for recorded snapshots. Uses custom serialization to store
    /// durations as nanoseconds instead of `[seconds, nanoseconds]` pairs.
    #[serde(with = "vec_duration")]
    pub record_elapsed_time: Vec<Duration>,
    pub replay_timestamps: Vec<DateTime<Utc>>,
    #[serde(with = "vec_duration")]
    pub replay_elapsed_time: Vec<Duration>,
    pub replay_results: Vec<ReplayResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ReplayResult {
    Pass,
    Fail,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                test_file: PathBuf::from(TEST_FILE),
                snapshot_dir: PathBuf::from(SNAPSHOT_DIR),
                history: HISTORY,
            },
            record: RecordConfig {
                overwrite: OVERWRITE,
            },
            replay: ReplayConfig {
                fail_fast: FAILFAST,
            },
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

    pub fn init(&self, base_dir: &Path, snapshot_dir: &Path, test_file: &Path) -> Result<()> {
        fs::create_dir_all(base_dir.join(snapshot_dir))
            .context("Failed to construct path to snapshot directory")?;
        fs::File::create(base_dir.join(test_file)).context("Unable to create test file")?;
        Ok(())
    }

    pub fn update_latest_record(
        &mut self,
        config_path: &PathBuf,
        snapshot_path: PathBuf,
        elapsed: Duration,
    ) -> Result<()> {
        let timestamp = Utc::now();

        self.state.latest_snapshots.insert(0, snapshot_path);
        self.state.record_timestamps.insert(0, timestamp);
        self.state.record_elapsed_time.insert(0, elapsed);

        // Trim to history limit
        let limit = self.common.history;
        self.state.latest_snapshots.truncate(limit);
        self.state.record_timestamps.truncate(limit);
        self.state.record_elapsed_time.truncate(limit);

        self.save(config_path)
    }

    pub fn update_latest_replay(
        &mut self,
        config_path: &PathBuf,
        elapsed: Duration,
        result: ReplayResult,
    ) -> Result<()> {
        let timestamp = Utc::now();

        self.state.replay_timestamps.insert(0, timestamp);
        self.state.replay_elapsed_time.insert(0, elapsed);
        self.state.replay_results.insert(0, result);

        // Trim to history limit
        let limit = self.common.history;
        self.state.replay_timestamps.truncate(limit);
        self.state.replay_elapsed_time.truncate(limit);
        self.state.replay_results.truncate(limit);

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

    config.init(
        config_path.parent().unwrap(),
        &config.common.snapshot_dir,
        &config.common.test_file,
    )?;
    config.save(config_path)?;
    Ok(config)
}

/// Wrapper struct that allows serializing and deserializing `chrono::Duration` values as a single
/// number of nanoseconds rather than the default `[seconds, nanoseconds]` format.
#[derive(Debug)]
pub struct SerializableDuration(Duration);

impl Serialize for SerializableDuration {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the `Duration` to nanoseconds and serlize as a single number
        let nanos = self.0.num_nanoseconds().unwrap_or(0) as u64;
        serializer.serialize_u64(nanos)
    }
}

impl<'de> Deserialize<'de> for SerializableDuration {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Read the nanoseconds value and convert back to a `Duration`
        let millis = u64::deserialize(deserializer)?;
        Ok(SerializableDuration(Duration::nanoseconds(millis as i64)))
    }
}

/// Module providing serialization for `Vec<Duration>`.
///
/// This module is used with serde's `#[serde(with = "vec_duration")]` attribute to customize how
/// vectors of `Duration` are serialized/deserialized.
mod vec_duration {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serializes a `Vec<Duration>` by converting each `Duration` to a `SerializableDuration`.
    pub fn serialize<S>(times: &Vec<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert each Duration to our SerializableDuration wrapper and serialize the vector
        let v: Vec<SerializableDuration> = times.iter().map(|d| SerializableDuration(*d)).collect();
        v.serialize(serializer)
    }

    /// Deserializes a `Vec<Duration>` by first deserializing to Vec<SerializableDuration>
    /// and then converting back to Vec<Duration>.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize to Vec<SerializableDuration> first, then convert to Vec<Duration>
        let v: Vec<SerializableDuration> = Vec::deserialize(deserializer)?;
        Ok(v.into_iter().map(|sd| sd.0).collect())
    }
}
