# ReRe.rs (Record Replay)

This project is a command line universal behavior testing tool inspired by [rere.py](https://github.com/tsoding/rere.py). I was inspired by the simple, elegant solution presented in the original project to the problem of universal testing, but wanted a compiled binary for more ubiquitous and comprehensive usage. This project provides a significant extension on top of the original concept. The rere tool makes use of the [bi format](https://github.com/tsoding/bi-format), a simple structured human readable binary format.

## Project Structure

The project is split into two parts:

1. [`bi-parser`](/bi-parser/README.md): A library crate providing functionality for validating, reading, and writing bi formatted files.
2. [`rere-app`](/rere-app/README.md): A binary crate which uses the `bi-parser` library to create the actual command line tool.

## Quick Start

1. Download the release binary or build the crate from source.
2. From your project's root directory, run `rere init`, this will create:
   - A config at `rere/rere.toml`
   - A directory to store the record snapshots at `rere/snapshots/`
   - A test file at `rere/test.list`
3. Add some shell commands to the test file (i.e. `echo "Hello World!`).
4. Record the expected behavior with `rere record`. This will create the `.bi` snapshot file here: `rere/snapshots/test.list.bi`.
5. Later, you can run `rere replay` to replay the test file (the default `fail_fast` setting is `true` so the replay will report the first difference, if any, and exit).

Much more comprehensive usage documentation can be found [here](/rere-app/README.md) .

Note: All these features are configurable either through [command line arguments](/rere-app/README.md#arguments) or by manually building/editing the `.toml` [config file](/rere-app/README.md#config-file).

## New Features

Several features over the original project were built into the rere.rs app, including:

- Command line tool
- Config file for persisting and tracking testing configurations
- Init command functionality for easy test structure setup and cleanup
- Dedicated snapshot and recording directory
- Ability to version snapshot files for testing history
- Basic metadata tracking
- Fast fail options for snapshot replaying
- Single line comment (`//`) support in test files

In addition, this project supports a minor extension on original the bi format:

- A signed integer marker type (`:s`)

### Planned Features

- Ability to capture file system changes (for testing scripts that write to log files instead of to stdout)
- Additional metadata tracking
- Warnings if significant deviation from previous processing times
- Standardized readme
- Tab completions
