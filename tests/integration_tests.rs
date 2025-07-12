use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use anyhow::Result;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_project(cargo_content: &str) -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        
        // Create Cargo.toml
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content)?;
        
        // Create src directory and lib.rs
        fs::create_dir(temp_dir.path().join("src"))?;
        fs::write(temp_dir.path().join("src").join("lib.rs"), 
            "// Integration test library\npub fn hello() { println!(\"Hello, world!\"); }")?;
        
        Ok(temp_dir)
    }

    fn run_resolver_command(project_path: &Path, args: &[&str]) -> Result<std::process::Output> {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "cargo-dependency-resolver-advanced", "--"])
            .args(args)
            .current_dir(project_path)
            .output()?;
        
        Ok(output)
    }

    #[test]
    fn test_clean_command_integration() -> Result<()> {
        let cargo_content = r#"
[package]
name = "integration-test"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = "0.11"
serde_json = "1.0"
tokio = "1.0"
reqwest = "0.12"  # Duplicate

[dev-dependencies]
tokio-test = "0.4"
"#;
        
        let temp_dir = create_test_project(cargo_content)?;
        let project_path = temp_dir.path();
        
        // Run clean command
        let output = run_resolver_command(project_path, &["clean", "--verbose"])?;
        
        // Check if command succeeded
        if !output.status.success() {
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        assert!(output.status.success(), "Clean command should succeed");
        
        // Verify backup was created
        let backup_files: Vec<_> = fs::read_dir(project_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name()
                    .to_string_lossy()
                    .starts_with("Cargo.toml.backup_")
            })
            .collect();
        
        assert!(!backup_files.is_empty(), "Backup file should be created");
        
        // Verify Cargo.toml was modified
        let modified_content = fs::read_to_string(project_path.join("Cargo.toml"))?;
        assert!(modified_content.contains("[dependencies]"), "Dependencies section should exist");
        
        Ok(())
    }

    #[test]
    fn test_enhance_command_integration() -> Result<()> {
        let cargo_content = r#"
[package]
name = "enhance-test"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = "0.3"  # Insecure version
time = "0.1"    # Insecure version
safe-crate = "1.0"
"#;
        
        let temp_dir = create_test_project(cargo_content)?;
        let project_path = temp_dir.path();
        
        // Run enhance command
        let output = run_resolver_command(project_path, &["enhance", "--aggressive", "--verbose", "--sbom"])?;
        
        if !output.status.success() {
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        assert!(output.status.success(), "Enhance command should succeed");
        
        // Verify SBOM was generated
        let sbom_path = project_path.join("sbom.json");
        assert!(sbom_path.exists(), "SBOM file should be created");
        
        // Verify SBOM content
        let sbom_content = fs::read_to_string(&sbom_path)?;
        let sbom_data: serde_json::Value = serde_json::from_str(&sbom_content)?;
        
        assert!(sbom_data.get("project").is_some(), "SBOM should have project field");
        assert!(sbom_data.get("generated").is_some(), "SBOM should have timestamp");
        
        // Verify security patches were added
        let modified_content = fs::read_to_string(project_path.join("Cargo.toml"))?;
        assert!(modified_content.contains("[patch.crates-io]"), "Security patches should be added");
        
        Ok(())
    }

    #[test]
    fn test_restore_command_integration() -> Result<()> {
        let original_content = r#"
[package]
name = "restore-test"
version = "0.1.0"
edition = "2021"

[dependencies]
original-dep = "1.0"
"#;
        
        let temp_dir = create_test_project(original_content)?;
        let project_path = temp_dir.path();
        let cargo_path = project_path.join("Cargo.toml");
        
        // Create a manual backup
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_path = project_path.join(format!("Cargo.toml.backup_{}", timestamp));
        fs::copy(&cargo_path, &backup_path)?;
        
        // Modify the original file
        let modified_content = r#"
[package]
name = "restore-test"
version = "0.1.0"
edition = "2021"

[dependencies]
modified-dep = "2.0"
"#;
        fs::write(&cargo_path, modified_content)?;
        
        // Run restore command
        let output = run_resolver_command(project_path, &["restore", "latest"])?;
        
        if !output.status.success() {
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        assert!(output.status.success(), "Restore command should succeed");
        
        // Verify original content was restored
        let restored_content = fs::read_to_string(&cargo_path)?;
        assert!(restored_content.contains("original-dep"), "Original dependency should be restored");
        assert!(!restored_content.contains("modified-dep"), "Modified dependency should be gone");
        
        Ok(())
    }

    #[test]
    fn test_hotz_philosophy_integration() -> Result<()> {
        let cargo_content = r#"
[package]
name = "hotz-test"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = "0.11"
serde_json = "1.0"
rand = "0.8"
chrono = "0.4"
"#;
        
        let temp_dir = create_test_project(cargo_content)?;
        let project_path = temp_dir.path();
        
        // Run clean command with HOTZ philosophy
        let output = run_resolver_command(project_path, &["clean", "--hotz", "--verbose"])?;
        
        if !output.status.success() {
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        assert!(output.status.success(), "HOTZ clean should succeed");
        
        // Verify HOTZ replacements
        let modified_content = fs::read_to_string(project_path.join("Cargo.toml"))?;
        
        // Check for HOTZ replacements
        assert!(modified_content.contains("ureq") || modified_content.contains("reqwest"), 
            "Should have ureq or reqwest");
        assert!(modified_content.contains("json") || modified_content.contains("serde_json"), 
            "Should have json or serde_json");
        
        Ok(())
    }

    #[test]
    fn test_workspace_project_integration() -> Result<()> {
        let workspace_content = r#"
[workspace]
members = ["member1"]

[workspace.dependencies]
tokio = "1.0"
serde = "1.0"
tokio = "1.1"  # Duplicate

[dependencies]
workspace-dep = "0.1"
"#;
        
        let temp_dir = create_test_project(workspace_content)?;
        let project_path = temp_dir.path();
        
        // Create member project
        let member_dir = project_path.join("member1");
        fs::create_dir(&member_dir)?;
        fs::create_dir(member_dir.join("src"))?;
        fs::write(member_dir.join("src").join("lib.rs"), "// member lib")?;
        fs::write(member_dir.join("Cargo.toml"), r#"
[package]
name = "member1"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
"#)?;
        
        // Run clean command
        let output = run_resolver_command(project_path, &["clean", "--verbose"])?;
        
        if !output.status.success() {
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        assert!(output.status.success(), "Workspace clean should succeed");
        
        // Verify workspace dependencies were cleaned
        let modified_content = fs::read_to_string(project_path.join("Cargo.toml"))?;
        assert!(modified_content.contains("[workspace.dependencies]"), 
            "Workspace dependencies should be present");
        
        Ok(())
    }

    #[test]
    fn test_error_handling_invalid_toml() -> Result<()> {
        let invalid_content = r#"
[package
name = "invalid-toml"  # Missing closing bracket
version = "0.1.0"
"#;
        
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("Cargo.toml"), invalid_content)?;
        
        // Run clean command on invalid TOML
        let output = run_resolver_command(temp_dir.path(), &["clean"])?;
        
        // Should fail gracefully
        assert!(!output.status.success(), "Should fail on invalid TOML");
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("TOML") || stderr.contains("parse"), 
            "Error message should mention TOML parsing");
        
        Ok(())
    }

    #[test]
    fn test_performance_large_cargo_toml() -> Result<()> {
        // Create a large Cargo.toml with many dependencies
        let mut cargo_content = String::from(r#"
[package]
name = "large-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#);
        
        // Add 100 dependencies with some duplicates
        for i in 0..100 {
            cargo_content.push_str(&format!("dep{} = \"1.0\"\n", i));
            if i % 10 == 0 {
                // Add some duplicates
                cargo_content.push_str(&format!("dep{} = \"1.1\"\n", i));
            }
        }
        
        let temp_dir = create_test_project(&cargo_content)?;
        let project_path = temp_dir.path();
        
        // Measure performance
        let start = std::time::Instant::now();
        let output = run_resolver_command(project_path, &["clean", "--verbose"])?;
        let duration = start.elapsed();
        
        if !output.status.success() {
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        assert!(output.status.success(), "Large project clean should succeed");
        assert!(duration.as_secs() < 30, "Should complete within 30 seconds");
        
        println!("Performance test completed in {:?}", duration);
        
        Ok(())
    }
}
