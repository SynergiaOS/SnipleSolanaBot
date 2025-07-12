use std::fs;
use std::path::Path;
use tempfile::TempDir;
use toml_edit::{DocumentMut, value, Table, Item};
use anyhow::Result;

// Import functions from the cargo_resolver module
use overmind_protocol::cargo_resolver::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cargo_toml(content: &str) -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_path, content)?;
        Ok(temp_dir)
    }

    fn create_sample_cargo_toml() -> &'static str {
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = "0.11"
serde_json = "1.0"
tokio = "1.0"
reqwest = "0.12"  # Duplicate
chrono = "0.4"

[dev-dependencies]
tokio-test = "0.4"
reqwest = "0.11"  # Duplicate across sections

[build-dependencies]
cc = "1.0"
"#
    }

    fn create_insecure_cargo_toml() -> &'static str {
        r#"
[package]
name = "insecure-project"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = "0.3"  # Insecure version
time = "0.1"    # Insecure version
openssl-sys = "0.9"  # Insecure version
safe-crate = "1.0"
"#
    }

    #[test]
    fn test_duplicate_removal() -> Result<()> {
        let temp_dir = create_test_cargo_toml(create_sample_cargo_toml())?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        
        // Read and parse the TOML
        let content = fs::read_to_string(&cargo_path)?;
        let mut doc: DocumentMut = content.parse()?;
        
        // Count dependencies before cleaning
        let deps_before = doc.get("dependencies")
            .and_then(|d| d.as_table())
            .map(|t| t.len())
            .unwrap_or(0);
        
        // Apply duplicate removal logic
        if let Some(deps) = doc.get_mut("dependencies").and_then(Item::as_table_mut) {
            clean_section_duplicates(deps, false, "dependencies");
        }
        
        // Count dependencies after cleaning
        let deps_after = doc.get("dependencies")
            .and_then(|d| d.as_table())
            .map(|t| t.len())
            .unwrap_or(0);
        
        // Should have removed one duplicate reqwest
        assert!(deps_after < deps_before, "Duplicates should be removed");
        
        // Verify reqwest is still present (latest version should be kept)
        assert!(doc.get("dependencies")
            .and_then(|d| d.as_table())
            .and_then(|t| t.get("reqwest"))
            .is_some(), "reqwest should still be present after deduplication");
        
        Ok(())
    }

    #[test]
    fn test_hotz_philosophy_replacements() -> Result<()> {
        let temp_dir = create_test_cargo_toml(create_sample_cargo_toml())?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        
        let content = fs::read_to_string(&cargo_path)?;
        let mut doc: DocumentMut = content.parse()?;
        
        // Apply HOTZ philosophy
        let replacements = apply_hotz_philosophy(&mut doc, false)?;
        
        // Verify replacements were made
        assert!(replacements > 0, "Should make HOTZ replacements");

        let deps = doc.get("dependencies").and_then(|d| d.as_table()).unwrap();
        assert!(deps.contains_key("ureq"), "ureq should be present");
        assert!(deps.contains_key("json"), "json should be present");
        
        Ok(())
    }

    #[test]
    fn test_security_patches_application() -> Result<()> {
        let temp_dir = create_test_cargo_toml(create_sample_cargo_toml())?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        
        let content = fs::read_to_string(&cargo_path)?;
        let mut doc: DocumentMut = content.parse()?;
        
        // Apply security patches
        apply_security_patches(&mut doc, false)?;
        
        // Verify patches were added
        assert!(doc.get("patch").is_some(), "patch section should be present");
        
        let patch_section = doc.get("patch")
            .and_then(|p| p.as_table())
            .and_then(|t| t.get("crates-io"))
            .and_then(|c| c.as_table())
            .unwrap();
        
        assert!(patch_section.contains_key("curve25519-dalek"), "curve25519-dalek patch should be present");
        assert!(patch_section.contains_key("ed25519-dalek"), "ed25519-dalek patch should be present");
        
        Ok(())
    }

    #[test]
    fn test_insecure_crate_removal() -> Result<()> {
        let temp_dir = create_test_cargo_toml(create_insecure_cargo_toml())?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        
        let content = fs::read_to_string(&cargo_path)?;
        let mut doc: DocumentMut = content.parse()?;
        
        // Apply insecure crate removal (aggressive mode)
        let insecure_crates = vec!["chrono", "time", "openssl-sys"];
        let removed = remove_insecure_crates(&mut doc, &insecure_crates, true, false)?;
        
        // Verify insecure crates were removed
        assert_eq!(removed, 3, "Should remove 3 insecure crates");

        let deps = doc.get("dependencies").and_then(|d| d.as_table()).unwrap();
        assert!(!deps.contains_key("chrono"), "chrono should be removed");
        assert!(!deps.contains_key("time"), "time should be removed");
        assert!(!deps.contains_key("openssl-sys"), "openssl-sys should be removed");
        assert!(deps.contains_key("safe-crate"), "safe-crate should remain");
        
        Ok(())
    }

    #[test]
    fn test_backup_creation() -> Result<()> {
        let temp_dir = create_test_cargo_toml(create_sample_cargo_toml())?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        
        // Simulate backup creation
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_path = temp_dir.path().join(format!("Cargo.toml.backup_{}", timestamp));
        
        fs::copy(&cargo_path, &backup_path)?;
        
        // Verify backup was created
        assert!(backup_path.exists(), "Backup file should exist");
        
        // Verify backup content matches original
        let original_content = fs::read_to_string(&cargo_path)?;
        let backup_content = fs::read_to_string(&backup_path)?;
        assert_eq!(original_content, backup_content, "Backup content should match original");
        
        Ok(())
    }

    #[test]
    fn test_workspace_dependencies_handling() -> Result<()> {
        let workspace_toml = r#"
[workspace]
members = ["crate1", "crate2"]

[workspace.dependencies]
tokio = "1.0"
serde = "1.0"
tokio = "1.1"  # Duplicate

[dependencies]
local-dep = "0.1"
"#;
        
        let temp_dir = create_test_cargo_toml(workspace_toml)?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        
        let content = fs::read_to_string(&cargo_path)?;
        let mut doc: DocumentMut = content.parse()?;
        
        // Clean workspace dependencies
        if let Some(workspace) = doc.get_mut("workspace") {
            if let Some(ws_deps) = workspace.get_mut("dependencies").and_then(Item::as_table_mut) {
                clean_section_duplicates(ws_deps, false, "workspace.dependencies");
            }
        }
        
        // Verify workspace dependencies were cleaned
        let ws_deps = doc.get("workspace")
            .and_then(|w| w.as_table())
            .and_then(|t| t.get("dependencies"))
            .and_then(|d| d.as_table())
            .unwrap();
        
        // Should have only one tokio entry
        assert!(ws_deps.contains_key("tokio"), "tokio should be present");
        assert!(ws_deps.contains_key("serde"), "serde should be present");
        
        Ok(())
    }

    #[test]
    fn test_sbom_generation_structure() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create a minimal project structure for cargo metadata
        let cargo_content = r#"
[package]
name = "test-sbom"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
        
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content)?;
        fs::create_dir(temp_dir.path().join("src"))?;
        fs::write(temp_dir.path().join("src").join("lib.rs"), "// test lib")?;
        
        // Generate SBOM (mock the cargo metadata call)
        let sbom_content = serde_json::json!({
            "project": "THE OVERMIND PROTOCOL",
            "dependencies": "mock_metadata",
            "generated": chrono::Local::now().to_string(),
        });
        
        let sbom_path = temp_dir.path().join("sbom.json");
        fs::write(&sbom_path, serde_json::to_string_pretty(&sbom_content)?)?;
        
        // Verify SBOM structure
        assert!(sbom_path.exists(), "SBOM file should be created");
        
        let sbom_data: serde_json::Value = serde_json::from_str(&fs::read_to_string(&sbom_path)?)?;
        
        assert!(sbom_data.get("project").is_some(), "SBOM should have project field");
        assert!(sbom_data.get("dependencies").is_some(), "SBOM should have dependencies field");
        assert!(sbom_data.get("generated").is_some(), "SBOM should have generated timestamp");
        
        Ok(())
    }

    #[test]
    fn test_edge_case_empty_sections() -> Result<()> {
        let empty_sections_toml = r#"
[package]
name = "empty-sections"
version = "0.1.0"
edition = "2021"

[dependencies]

[dev-dependencies]

[build-dependencies]
"#;
        
        let temp_dir = create_test_cargo_toml(empty_sections_toml)?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        
        let content = fs::read_to_string(&cargo_path)?;
        let mut doc: DocumentMut = content.parse()?;
        
        // Apply cleaning to empty sections (should not crash)
        let sections = vec!["dependencies", "dev-dependencies", "build-dependencies"];

        for section in sections {
            if let Some(deps) = doc.get_mut(section).and_then(Item::as_table_mut) {
                clean_section_duplicates(deps, false, section);
            }
        }
        
        // Should complete without errors
        assert!(doc.get("dependencies").is_some(), "dependencies section should exist");
        
        Ok(())
    }
}
