use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn rere_cmd() -> Command {
    Command::cargo_bin("rere").unwrap()
}

#[test]
fn test_init_command() {
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("rere/rere.toml");
    let test_file = "custom_test.list";
    let snapshot_dir = "custom_snapshots";

    let output = rere_cmd()
        .arg(&config_path)
        .arg("init")
        .arg("--test-file")
        .arg(test_file)
        .arg("--snapshot-dir")
        .arg(snapshot_dir)
        .arg("--history")
        .arg("5")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("Initialized config at {}", config_path.display())));
    assert!(output.status.success());

    // Verify directory structure
    assert!(config_path.exists());
    assert!(temp.path().join("rere").join(snapshot_dir).exists());
    assert!(temp.path().join("rere").join(test_file).exists());

    // Verify config content
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains(test_file));
    assert!(config_content.contains(snapshot_dir));
    assert!(config_content.contains("history = 5"));
}

#[test]
fn test_record_command() {
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("rere/rere.toml");

    // Initialize first
    rere_cmd()
        .arg(&config_path)
        .arg("init")
        .output()
        .unwrap();

    // Create test.list with some commands
    let test_list_path = temp.path().join("rere/test.list");
    fs::write(&test_list_path, "echo 'test'\nls -la\n").unwrap();

    // Record
    let output = rere_cmd()
        .arg(&config_path)
        .arg("record")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Recording completed successfully"));
    assert!(output.status.success());

    // Verify snapshot was created
    let snapshot_path = temp.path().join("rere/snapshots/test.list.bi");
    assert!(snapshot_path.exists());
}

#[test]
fn test_clean_snapshots() {
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("rere/rere.toml");

    // Initialize and create a snapshot
    rere_cmd()
        .arg(&config_path)
        .arg("init")
        .output()
        .unwrap();

    let test_list_path = temp.path().join("rere/test.list");
    fs::write(&test_list_path, "echo 'test'\n").unwrap();

    rere_cmd()
        .arg(&config_path)
        .arg("record")
        .output()
        .unwrap();

    // Clean snapshots with auto-confirmation
    let output = rere_cmd()
        .arg(&config_path)
        .arg("clean")
        .arg("--snapshots")
        .write_stdin("y\n")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify snapshots directory is empty but exists
    let snapshot_dir = temp.path().join("rere/snapshots");
    assert!(snapshot_dir.exists());
    assert!(fs::read_dir(&snapshot_dir).unwrap().next().is_none());
}

#[test]
fn test_clean_all() {
    let temp = tempdir().unwrap();

    // Initialize
    rere_cmd()
        .arg("init")
        .output()
        .unwrap();

    // Clean all with auto-confirmation
    let output = rere_cmd()
        .arg("clean")
        .arg("--all")
        .write_stdin("y\n")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify rere directory no longer exists
    assert!(!temp.path().join("rere").exists());
}
