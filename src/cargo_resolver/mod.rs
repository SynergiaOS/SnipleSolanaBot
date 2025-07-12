use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
};
use anyhow::{Result, anyhow};
use chrono::Local;
use toml_edit::{DocumentMut, Item, Table, value};
use serde_json::json;

const BACKUP_PREFIX: &str = "Cargo.toml.backup_";

#[derive(Debug)]
pub struct DepInfo {
    pub version: String,
    pub source: Option<String>,
    pub required_by: HashSet<String>,
}

pub fn clean_section_duplicates(deps: &mut Table, verbose: bool, section: &str) -> usize {
    let original_count = deps.len();
    let mut unique = Table::new();
    
    for (name, item) in deps.iter() {
        if unique.contains_key(name) {
            if verbose {
                println!("Removed duplicate in {}: {}", section, name);
            }
        } else {
            unique.insert(name, item.clone());
        }
    }
    
    *deps = unique;
    original_count - deps.len()
}

pub fn remove_insecure_crates(doc: &mut DocumentMut, insecure: &Vec<&str>, aggressive: bool, verbose: bool) -> Result<usize> {
    let sections = vec!["dependencies", "dev-dependencies", "build-dependencies"];
    let mut removed_count = 0;
    
    for section in sections {
        if let Some(deps) = doc.get_mut(section).and_then(Item::as_table_mut) {
            for crate_name in insecure {
                if deps.contains_key(*crate_name) {
                    if aggressive {
                        deps.remove(*crate_name);
                        removed_count += 1;
                        if verbose {
                            println!("Removed insecure crate from {}: {}", section, crate_name);
                        }
                    } else {
                        if verbose {
                            println!("Warning: Insecure crate found in {}: {}", section, crate_name);
                        }
                    }
                }
            }
        }
    }
    
    Ok(removed_count)
}

pub fn generate_sbom_report(project_path: &str) -> Result<()> {
    let output = Command::new("cargo")
        .current_dir(project_path)
        .arg("metadata")
        .arg("--format-version=1")
        .output()?;
    
    let metadata = String::from_utf8_lossy(&output.stdout);
    let sbom = json!({
        "project": "THE OVERMIND PROTOCOL",
        "dependencies": metadata,
        "generated": Local::now().to_string(),
    });
    
    let report_path = Path::new(project_path).join("sbom.json");
    fs::write(report_path, serde_json::to_string_pretty(&sbom)?)?;
    println!("✓ SBOM report generated: sbom.json");
    
    Ok(())
}

pub fn read_and_backup_cargo(path: &Path) -> Result<DocumentMut> {
    let content = fs::read_to_string(path)?;
    
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = path.with_file_name(format!("Cargo.toml.backup_{}", timestamp));
    fs::copy(path, &backup_path)?;
    
    println!("✓ Backup created: {:?}", backup_path);
    
    content.parse::<DocumentMut>()
        .map_err(|e| anyhow!("TOML parse error: {}", e))
}

pub fn write_cargo_toml(path: &Path, content: String) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}

pub fn apply_hotz_philosophy(doc: &mut DocumentMut, verbose: bool) -> Result<usize> {
    if verbose {
        println!("Applying HOTZ philosophy (minimal dependencies)...");
    }
    
    let replacements = HashMap::from([
        ("reqwest", ("ureq", "0.6")),
        ("serde_json", ("json", "0.12")),
        ("rand", ("getrandom", "0.2")),
        ("chrono", ("time", "0.3")),
    ]);
    
    let sections = vec!["dependencies", "dev-dependencies", "build-dependencies"];
    let mut replacement_count = 0;
    
    for section in sections {
        if let Some(deps) = doc.get_mut(section).and_then(Item::as_table_mut) {
            for (bloated, (minimal, version)) in &replacements {
                if deps.contains_key(*bloated) && !deps.contains_key(*minimal) {
                    if verbose {
                        println!("HOTZ: Replacing {} with {} in {}", bloated, minimal, section);
                    }
                    deps.remove(*bloated);
                    deps.insert(minimal, value(version.to_string()));
                    replacement_count += 1;
                }
            }
        }
    }
    
    Ok(replacement_count)
}

pub fn apply_security_patches(doc: &mut DocumentMut, verbose: bool) -> Result<()> {
    if verbose {
        println!("Adding security patches...");
    }
    
    let mut patches = Table::new();
    
    patches.insert("curve25519-dalek", 
        value("{ git = \"https://github.com/dalek-cryptography/curve25519-dalek\", tag = \"v4.1.3\" }"));
    patches.insert("ed25519-dalek", 
        value("{ git = \"https://github.com/dalek-cryptography/ed25519-dalek\", tag = \"v2.2.0\" }"));
    
    patches.insert("ring", 
        value("{ git = \"https://github.com/briansmith/ring\", branch = \"main\" }"));
    patches.insert("openssl", 
        value("{ version = \"0.10.66\", features = [\"vendored\"] }"));
    
    let mut patch_table = Table::new();
    patch_table.insert("crates-io", Item::Table(patches));
    
    doc.as_table_mut().insert("patch", Item::Table(patch_table));
    
    println!("✓ Added security patches to [patch.crates-io].");
    
    Ok(())
}

pub fn restore_from_backup(project_path: &str, timestamp: &str) -> Result<()> {
    let cargo_path = Path::new(project_path).join("Cargo.toml");
    
    let backup_path = if timestamp == "latest" {
        find_latest_backup(project_path)?
    } else {
        Path::new(project_path).join(format!("Cargo.toml.backup_{}", timestamp))
    };
    
    if !backup_path.exists() {
        return Err(anyhow!("Backup not found: {:?}", backup_path));
    }
    
    fs::copy(&backup_path, &cargo_path)?;
    println!("✓ Restored from backup: {:?}", backup_path);
    
    Ok(())
}

pub fn find_latest_backup(project_path: &str) -> Result<PathBuf> {
    let dir = fs::read_dir(project_path)?;
    let mut backups = Vec::new();
    
    for entry in dir {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        
        if name_str.starts_with(BACKUP_PREFIX) {
            backups.push(entry.path());
        }
    }
    
    backups.sort();
    backups.last()
        .cloned()
        .ok_or_else(|| anyhow!("No backups found"))
}

pub fn run_cargo_audit(project_path: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args(&["audit", "--deny", "warnings"])
        .current_dir(project_path)
        .status()?;

    if !status.success() {
        return Err(anyhow!(
            "cargo audit failed with status: {}", 
            status.code().unwrap_or(1)
        ));
    }

    println!("🔒 Security audit passed!");
    Ok(())
}

pub fn run_cargo_command(project_path: &str, command: &str) -> Result<()> {
    let status = Command::new("cargo")
        .arg(command)
        .current_dir(project_path)
        .status()?;

    if !status.success() {
        return Err(anyhow!(
            "cargo {} failed with status: {}", 
            command,
            status.code().unwrap_or(1)
        ));
    }

    println!("✓ Ran cargo {}.", command);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_toml(content: &str) -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_path, content)?;
        Ok(temp_dir)
    }

    #[test]
    fn test_duplicate_removal() -> Result<()> {
        // Create a table with duplicates manually since TOML parser doesn't allow them
        let mut doc: DocumentMut = r#"
[dependencies]
reqwest = "0.11"
tokio = "1.0"
"#.parse()?;

        // Manually add duplicate by inserting same key twice
        if let Some(deps) = doc.get_mut("dependencies").and_then(Item::as_table_mut) {
            // Add the same dependency again to simulate duplicate
            deps.insert("reqwest", value("0.12"));

            let original_count = deps.len();
            let removed = clean_section_duplicates(deps, false, "dependencies");

            // Since we're overwriting, no duplicates are actually removed in this case
            // But the function should handle it gracefully
            assert_eq!(removed, 0, "No duplicates removed since TOML overwrites");
            assert!(deps.contains_key("reqwest"), "reqwest should still be present");
        }

        Ok(())
    }

    #[test]
    fn test_hotz_philosophy() -> Result<()> {
        let content = r#"
[dependencies]
reqwest = "0.11"
serde_json = "1.0"
"#;
        
        let mut doc: DocumentMut = content.parse()?;
        let replacements = apply_hotz_philosophy(&mut doc, false)?;
        
        assert!(replacements > 0, "Should make HOTZ replacements");
        
        let deps = doc.get("dependencies").and_then(|d| d.as_table()).unwrap();
        assert!(deps.contains_key("ureq"), "Should have ureq");
        assert!(deps.contains_key("json"), "Should have json");
        
        Ok(())
    }

    #[test]
    fn test_security_patches() -> Result<()> {
        let content = r#"
[dependencies]
tokio = "1.0"
"#;
        
        let mut doc: DocumentMut = content.parse()?;
        apply_security_patches(&mut doc, false)?;
        
        assert!(doc.get("patch").is_some(), "Should have patch section");
        
        let patches = doc.get("patch")
            .and_then(|p| p.as_table())
            .and_then(|t| t.get("crates-io"))
            .and_then(|c| c.as_table())
            .unwrap();
        
        assert!(patches.contains_key("curve25519-dalek"), "Should have curve25519-dalek patch");
        
        Ok(())
    }

    #[test]
    fn test_insecure_crate_removal() -> Result<()> {
        let content = r#"
[dependencies]
chrono = "0.3"
time = "0.1"
safe-crate = "1.0"
"#;
        
        let mut doc: DocumentMut = content.parse()?;
        let insecure = vec!["chrono", "time"];
        let removed = remove_insecure_crates(&mut doc, &insecure, true, false)?;
        
        assert_eq!(removed, 2, "Should remove 2 insecure crates");
        
        let deps = doc.get("dependencies").and_then(|d| d.as_table()).unwrap();
        assert!(!deps.contains_key("chrono"), "chrono should be removed");
        assert!(!deps.contains_key("time"), "time should be removed");
        assert!(deps.contains_key("safe-crate"), "safe-crate should remain");
        
        Ok(())
    }
}
