use std::error::Error;

pub fn execute_command(command: &str) -> Result<String, Box<dyn Error>> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()?;

    let result = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        format!(
            "Command failed with exit code {:?}:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
    };

    Ok(result)
}
