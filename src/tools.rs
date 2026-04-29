use std::error::Error;

pub fn execute_command(command: &str) -> Result<String, Box<dyn Error>> {
    println!("[rcode] execute_command: {}", command);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_command_returns_output() {
        let result = execute_command("echo hello").unwrap();
        assert_eq!(result.trim(), "hello");
    }

    #[test]
    fn invalid_command_returns_error_message() {
        let result = execute_command("nonexistent_command_12345").unwrap();
        assert!(result.contains("not found"));
    }
}
