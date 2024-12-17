# ReRe.rs

The `rere` tool is a command line universal behavior testing tool for recording and replaying command outputs for behavior testing, inspired by [rere.py](https://github.com/tsoding/rere.py).

- [Installation](#installation)
  - [Release Binary](#release-binary)
  - [Building From Source](#building-from-source)
- [Usage](#usage)
  - [Initializing a Testing Environment](#initializing-a-testing-environment)
  - [Test File](#test-file)
  - [Recording](#recording)
  - [Replaying](#replaying)
  - [Cleaning Up Testing Environments](#cleaning-up-testing-environments)
- [Config File](#config-file)
  - [Common Table](#common-table)
  - [Record Table](#record-table)
  - [Replay Table](#replay-table)
  - [State Table](#state-table)
- [Arguments](#arguments)
  - [Init Arguments](#init-arguments)
  - [Clean Arguments](#clean-arguments)

---

## Installation

To download and use the rere command line tool, you have two options: downloading the release binary or compile from source.

### Release Binary

To download a release binary, go to the [releases](https://github.com/seankim658/rere.rs/releases) and download the binary for your OS.

### Building From Source

To build from source, you will need [git](https://git-scm.com/downloads), [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html), and Cargo (will be installed with Rust).

First clone the repository:

```
git clone git@github.com:seankim658/rere.rs.git
```

And then compile a release binary:

```
cd rere.rs
cargo build -p rere --release
```

## Usage

### Initializing a Testing Environment

To create a new testing environment, you can use the `init` subcommand like so:

```
rere init
```

This will create the following test structure:

```
rere
├── rere.toml
├── snapshots
└── test.list
```

The default test environment directory is `rere/`. The `rere.toml` file is the default configuration file, `snapshots/` is the default directory where recorded snapshots will be saved, and `test.list` is the default test file. The default names and paths can all be overridden through command line arguments. For further documentation on the command line options, see the [init arguments](#init-arguments) section.

### Test File

The test file is just a text file with one shell command per line. The test file also supports single line comments beginning with `//`. For example:

```
// This line is a comment and will be ignored during recording
echo "Hello World!"
ls
```

### Recording

Recording can be completed with the `record` subcommand: `rere record`

```bash
$ rere record
Capturing: echo "Hello World!"
Recording completed successfully
```

The resulting snapshot file will look like this:

```
:i count 1
:b shell 19
echo "Hello World!"
:s returncode 0
:b stdout 13
Hello World!

:b stderr 0

```

Note:
- The empty newline at the bottom of the file is part of the bi format, be careful if you manually edit the `.bi` files (not recommended to do so).
  - Important to remember that although the bi format is human readable, it is a binary format. Manually editing any bi files will most likely have unintended consequences.
- The snapshot file uses the `:s` (signed integer) field marker, which is introduced by this project and not described in the original bi format specification.

### Replaying

Recording can be completed with the `replay` subcommand: `rere replay`

```bash
$ rere replay
Replaying: echo "Hello World!"
Capturing: echo "Hello World!"
All tests passed!
Recording completed successfully
```

For examples sake, we can delete the `!` and view the output of a failed replay attempt:

```bash
$ rere replay
Replaying: echo "Hello World"

Unexpected shell command:
  Expected: echo "Hello World!" -> Actual: echo "Hello World"
Error during recording: Replay failed
```

In this case, the command was never executed as rere found a mismatched shell command during preprocessing.

We can try a command that will pass preprocessing checks but fail during execution. For this example, we use `ls` in the test file. Before replaying, I created a new file called `test.txt`. The replay output looks like this:

```bash
$ rere replay
Replaying: ls
Capturing: ls

Unexpected stdout:
  Expected:
    Cargo.lock
    Cargo.toml
    README.md
    bi-parser
    notes.md
    rere
    rere-app
    target
    <missing> -> test.txt
Error during recording: Replay failed
```

Here, the output tells us that we previously did not have a `test.txt` file (as denoted by the `<missing>`).

### Cleaning Up Testing Environments

If you want to clean up your testing environment, you can use the `clean` subcommand. The `clean` subcommand can be used to:

- Reset your config file to the default settings (note this will not reset your `test_file` and `snapshots_dir` routes)
- Clear out your existing snapshots
- Delete the entire testing environment

For further documentation on the command line options, see the [clean arguments](#clean-arguments) section.

## Config File

The configuration file is a simple `.toml` file, allowing for persistant test settings and basic metadata/history tracking of your snapshots.

### Common Table

The `[common]` table represents basic global level configurations for your testing environment.

- The `test_file` value represents the path, relative to the config file directory, to the test file to use for record and replay operations.
- Similarly, the `snapshots_dir` value represents the path, relative to the config file directory, to store the resulting snapshot `.bi` files.
- The `history` value controls how many snapshots or replays to keep data for.

```
[common]
test_file = "test.list"
snapshot_dir = "snapshots"
history = 3
```

### Record Table

The `[record]` table contains the currently supported record options.

- The `overwrite` value determines whether the resulting snapshot files will be overwritten or if a new snapshot file will be saved on each record run. If `overwrite` is set to `true`, the resulting snapshot file will be named `{test_file}.bi`, and that file will be overwritten on each recording. If `overwrite` is set to `false`, the snapshot file names will be formatted like so `{test_file}_{timestamp}.bi`. In this case, the maximum number of snapshot files saved at any time will be equal to the `history` value in the common table.

```
[record]
overwrite = true
```

### Replay Table

The `[replay]` table contains the currently supported replay options.

- The `fail_fast` value determines whether the replay operation should immediately exit on the first failure or continue.

```
[replay]
fail_fast = true
```

### State Table

The `[state]` table contains basic metadata and state information for the current testing environment. All state tracking will hold a maximum number of values equal to the `history` value in the common table. Additionally, all values are sorted with the most recent value occurring at index `0`.

- The `latest_snapshots` value holds the filenames of the latest snapshots. This value is used when `overwrite` is set to `false`.
- The `record_timestamps` value holds the timestamps of when each record operation was initiated.
- The `record_elapsed_time` value holds the elapsed time to complete the recording processes.
- The `replay_timestamps` value holds the timestamps of when each replay operation was initiated.
- The `replay_elapsed_time` value holds the elapsed time to complete the replay processes.
- The `replay_results` value holds the results (`pass` of `fail`) of each replay.
- In the case of a replay failing, the `replay_diffs` values holds an array of the diffs found in the replay results compared to the record snapshot file.

```
[state]
latest_snapshots = []
record_timestamps = []
record_elapsed_time = []
replay_timestamps = []
replay_elapsed_time = []
replay_results = []
replay_diffs = []
```

## Arguments

```
Rere arguments

Usage: rere [CONFIG] <COMMAND>

Commands:
  record  Record shell command args
  replay  Replay and verify shell commands against recorded snapshot
  init    Initialize a new rere config
  clean   Clean up snapshots, all testing files, or reset config
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [CONFIG]  Path to config file [default: ./rere/rere.toml]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Init Arguments

```
Initialize a new rere config

Usage: rere init [OPTIONS]

Options:
      --test-file <FILE>       Override test file location relative to config file directory [default: test.list]
      --snapshot-dir <DIR>     Overwrite default snapshots location relative to config file directory [default: snapshots/]
      --history <NUM>          Override default number of snapshots to keep metadata history for [default: 3]
      --overwrite <OVERWRITE>  Set overwrite default for record command [default: true] [possible values: true, false]
      --fail-fast <FAIL_FAST>  Set fail-fast default for replay command [default: true] [possible values: true, false]
  -h, --help                   Print help
```

### Clean Arguments

```
Clean up snapshots, all testing files, or reset config

Usage: rere clean [OPTIONS]

Options:
      --all        Clean up all testing files, directories, and subdirectories
      --snapshots  Clean up all snapshot files
      --config     Reset config file to defaults (except for `test_file` and `snapshot_dir` values)
  -h, --help       Print help
```
