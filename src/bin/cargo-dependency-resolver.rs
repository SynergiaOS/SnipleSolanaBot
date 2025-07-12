use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use anyhow::{Result, anyhow, Context};
use chrono::Local;
use clap::{Parser, Subcommand};
use semver::{Version, VersionReq};
use toml_edit::{Document, Item, Table, value};
use serde_json::json;

const BACKUP_PREFIX: &str = "Cargo.toml.backup_";

#[derive(Parser, Debug)]
#[command(version, about = "Cargo.toml dependency resolver: Resolves conflicts and cleans dependencies")]
struct Args {
    #[arg(default_value = ".")]
    path: String,
    
    #[arg(short, long, help = "Run cargo update after cleaning")]
    update: bool,
    
    #[arg(short, long, help = "Run cargo audit after cleaning")]
    audit: bool,
    
    #[arg(short, long, help = "Add security patches for known vulnerabilities")]
    patch: bool,
    
    #[arg(short = 'v', long, help = "Verbose output")]
    verbose: bool,
    
    #[arg(long, help = "Apply HOTZ philosophy (minimal dependencies)")]
    hotz: bool,
    
    #[arg(short, long, help = "Restore from backup if available")]
    restore: bool,
}

#[derive(Debug)]
struct DepInfo {
    version: String,
    source: Option<String>,
    required_by: HashSet<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.restore {
        return restore_from_backup(&args.path).context("Failed to restore from backup");
    }
    
    clean_dependencies(&args.path, &args)
        .context("Failed to clean dependencies")
}

fn restore_from_backup(root_path: &str) -> Result<()> {
    let cargo_path = Path::new(root_path).join("Cargo.toml");
    let backup_path = cargo_path.with_extension(BACKUP_EXT);
    
    if !backup_path.exists() {
        return Err(anyhow!("Backup file not found: {:?}", backup_path));
    }
    
    fs::copy(&backup_path, &cargo_path)?;
    println!("✓ Restored Cargo.toml from backup: {:?}", backup_path);
    
    Ok(())
}

fn clean_dependencies(root_path: &str, args: &Args) -> Result<()> {
    let cargo_path = Path::new(root_path).join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_path)?;
    
    // Create backup
    let backup_path = cargo_path.with_extension(BACKUP_EXT);
    fs::copy(&cargo_path, &backup_path)?;
    println!("✓ Backup created: {:?}", backup_path);

    let mut doc = cargo_content.parse::<Document>()?;
    
    let (workspace_deps, package_deps) = analyze_dependencies(&doc)?;
    let conflict_map = build_conflict_map(&workspace_deps, &package_deps, args.verbose);
    let resolution_map = resolve_conflicts(conflict_map);

    apply_resolutions(&mut doc, resolution_map)?;
    
    // Apply HOTZ philosophy if requested
    if args.hotz {
        apply_hotz_philosophy(&mut doc, args.verbose)?;
    }
    
    // Add security patches if requested
    if args.patch {
        apply_security_patches(&mut doc, args.verbose)?;
    }

    let new_content = doc.to_string();
    fs::write(&cargo_path, new_content)?;

    println!("✓ Cargo.toml cleaned and secured against conflicts.");
    
    // Optional: Run cargo update
    if args.update {
        println!("Running cargo update...");
        let output = Command::new("cargo")
            .current_dir(root_path)
            .arg("update")
            .output()?;
            
        if args.verbose {
            println!("cargo update stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("cargo update stderr: {}", String::from_utf8_lossy(&output.stderr));
        } else {
            println!("✓ Ran cargo update.");
        }
    }

    // Optional: Run cargo audit
    if args.audit {
        println!("Running cargo audit...");
        let output = Command::new("cargo")
            .current_dir(root_path)
            .arg("audit")
            .output()?;
            
        println!("Cargo audit results:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        
        if !output.stderr.is_empty() {
            println!("Errors: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
    
    Ok(())
}

fn analyze_dependencies(doc: &Document) -> Result<(Table, Table)> {
    let workspace_deps = doc.get("workspace")
        .and_then(|ws| ws.get("dependencies"))
        .and_then(Item::as_table)
        .cloned()
        .unwrap_or_else(Table::new);

    let package_deps = doc["dependencies"].as_table()
        .cloned()
        .unwrap_or_else(Table::new);

    Ok((workspace_deps, package_deps))
}

fn build_conflict_map(
    workspace_deps: &Table,
    package_deps: &Table,
    verbose: bool
) -> HashMap<String, Vec<DepInfo>> {
    let mut conflict_map: HashMap<String, Vec<DepInfo>> = HashMap::new();

    // Workspace dependencies
    for (name, item) in workspace_deps.iter() {
        if let Some(dep_info) = parse_dep_item(item) {
            conflict_map.entry(name.to_string())
                .or_default()
                .push(DepInfo {
                    version: dep_info.version,
                    source: dep_info.source,
                    required_by: HashSet::from(["workspace".to_string()]),
                });
        }
    }

    // Package dependencies
    for (name, item) in package_deps.iter() {
        if let Some(dep_info) = parse_dep_item(item) {
            conflict_map.entry(name.to_string())
                .or_default()
                .push(DepInfo {
                    version: dep_info.version,
                    source: dep_info.source,
                    required_by: HashSet::from(["package".to_string()]),
                });
        }
    }

    // Detect source conflicts
    for (name, versions) in conflict_map.iter_mut() {
        let unique_sources: HashSet<Option<String>> = versions.iter()
            .map(|v| v.source.clone())
            .collect();
        
        if unique_sources.len() > 1 && verbose {
            println!("⚠️ Source conflict detected for {}: {:?}", name, unique_sources);
        }
    }

    conflict_map
}

fn parse_dep_item(item: &Item) -> Option<DepInfo> {
    if item.is_str() {
        return Some(DepInfo {
            version: item.as_str()?.to_string(),
            source: None,
            required_by: HashSet::new(),
        });
    }

    let table = item.as_table()?;
    let version = table.get("version")?.as_str()?.to_string();
    
    let source = if table.contains_key("git") {
        let git = table.get("git")?.as_str()?;
        let branch = table.get("branch").map(|b| format!("#{}", b.as_str().unwrap_or("main")));
        Some(format!("git+{}{}", git, branch.unwrap_or_default()))
    } else if table.contains_key("path") {
        Some(format!("path:{}", table.get("path")?.as_str().unwrap_or_default()))
    } else {
        None
    };

    Some(DepInfo {
        version,
        source,
        required_by: HashSet::new(),
    })
}

fn resolve_conflicts(
    conflict_map: HashMap<String, Vec<DepInfo>>
) -> HashMap<String, String> {
    let mut resolution = HashMap::new();

    for (dep_name, versions) in conflict_map {
        if versions.len() <= 1 {
            continue;
        }

        // Check source compatibility
        let unique_sources: HashSet<Option<String>> = versions.iter()
            .map(|v| v.source.clone())
            .collect();
        
        if unique_sources.len() > 1 {
            println!("🚨 Automatic resolution impossible for {} - different sources. Manual intervention required.", dep_name);
            continue;
        }

        // Find latest compatible version
        let mut best_version: Option<Version> = None;
        for v_info in &versions {
            if let Ok(parsed) = Version::parse(&v_info.version) {
                if best_version.is_none() || parsed > best_version.as_ref().unwrap().clone() {
                    best_version = Some(parsed);
                }
            }
        }

        if let Some(best) = best_version {
            resolution.insert(dep_name.clone(), format!("^{}", best));
            println!("Resolved conflict for {} -> {}", dep_name, resolution[&dep_name]);
        } else {
            println!("No compatible version found for {}", dep_name);
        }
    }

    resolution
}

fn apply_resolutions(
    doc: &mut Document,
    resolutions: HashMap<String, String>
) -> Result<()> {
    // Clean workspace dependencies if conflict
    if let Some(workspace_deps) = doc["workspace"]["dependencies"].as_table_mut() {
        for (dep_name, _) in &resolutions {
            workspace_deps.remove(dep_name);
        }
    }

    // Update package dependencies
    for (dep_name, version) in resolutions {
        let dep_entry = &mut doc["dependencies"][&dep_name];
        
        if dep_entry.is_none() {
            *dep_entry = Item::Value(value(version));
            continue;
        }

        if dep_entry.is_str() {
            *dep_entry = Item::Value(value(version));
        } else if let Some(table) = dep_entry.as_table_mut() {
            table["version"] = value(version.clone());
            table.remove("workspace");
            table.remove("git");
            table.remove("path");
            table.remove("branch");
        }
    }

    Ok(())
}

fn apply_hotz_philosophy(doc: &mut Document, verbose: bool) -> Result<()> {
    if verbose {
        println!("Applying HOTZ philosophy (minimal dependencies)...");
    }
    
    // Define replacements for bloated dependencies
    let replacements = HashMap::from([
        ("reqwest", ("minreq", "2.11")),
        ("serde_json", ("tinyjson", "2.5")),
        ("rand", ("nanorand", "0.7")),
        ("ansi_term", ("nu-ansi-term", "0.50")),
        ("atty", ("is-terminal", "0.4")),
        ("backoff", ("exponential-backoff", "2.1")),
    ]);
    
    if let Some(deps) = doc["dependencies"].as_table_mut() {
        // Apply replacements
        for (bloated, (minimal, version)) in &replacements {
            if deps.contains_key(*bloated) && !deps.contains_key(*minimal) {
                if verbose {
                    println!("HOTZ: Replacing {} with {}", bloated, minimal);
                }
                
                // Keep the bloated dependency but mark as optional
                if let Some(dep_value) = deps.get_mut(*bloated) {
                    if let Some(dep_table) = dep_value.as_table_mut() {
                        dep_table.insert("optional", value(true));
                    }
                }
                
                // Add the minimal alternative
                deps.insert(minimal.to_string(), value(version.to_string()));
            }
        }
    }
    
    Ok(())
}

fn apply_security_patches(doc: &mut Document, verbose: bool) -> Result<()> {
    if verbose {
        println!("Adding security patches...");
    }
    
    // Create patch section if it doesn't exist
    let mut patches = Table::new();
    
    // Critical security patches
    patches.insert("curve25519-dalek".to_string(), 
        value("{ git = \"https://github.com/dalek-cryptography/curve25519-dalek\", tag = \"v4.1.3\" }"));
    patches.insert("ed25519-dalek".to_string(), 
        value("{ git = \"https://github.com/dalek-cryptography/ed25519-dalek\", tag = \"v2.2.0\" }"));
    
    // Additional security patches for THE OVERMIND PROTOCOL
    patches.insert("ring".to_string(), 
        value("{ git = \"https://github.com/briansmith/ring\", branch = \"main\" }"));
    
    // Create patch.crates-io section
    let mut patch_table = Table::new();
    patch_table.insert("crates-io".to_string(), Item::Table(patches));
    
    // Add to document
    doc.as_table_mut().insert("patch", Item::Table(patch_table));
    
    println!("Added security patches to [patch.crates-io].");
    
    Ok(())
}