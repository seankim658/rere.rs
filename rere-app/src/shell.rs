use anyhow::Result;
use std::process::Command;

pub struct CommandOutput {
    pub shell: String,
    pub returncode: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn capture(shell: &str) -> Result<CommandOutput> {
    println!("Capturing: {}", shell);

    let output = Command::new("sh").arg("-c").arg(shell).output()?;

    Ok(CommandOutput {
        shell: shell.to_owned(),
        returncode: output.status.code().unwrap_or(-1),
        stdout: output.stdout.clone(),
        stderr: output.stderr.clone(),
    })
}
