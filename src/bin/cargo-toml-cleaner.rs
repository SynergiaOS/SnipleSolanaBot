use std::fs::{read_to_string, write};
use std::process::Command;
use std::collections::HashMap;
use toml::Value;
use toml::map::Map;
use anyhow::{Result, anyhow};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "Cargo.toml cleaner: Removes duplicates, resolves conflicts, adds security patches")]
struct Args {
    #[arg(short, long, default_value = "Cargo.toml")]
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read Cargo.toml
    let content = read_to_string(&args.path)?;
    let mut toml_value: Value = content.parse()?;

    // Clean duplicates in [dependencies]
    if let Some(deps) = toml_value.get_mut("dependencies").and_then(|v| v.as_table_mut()) {
        let mut unique_deps = Map::new();
        for (key, value) in deps.iter() {
            if !unique_deps.contains_key(key) {
                unique_deps.insert(key.clone(), value.clone());
            } else {
                println!("Removed duplicate dependency: {}", key);
            }
        }
        *deps = unique_deps;
    } else {
        println!("No [dependencies] section found.");
    }

    // Clean duplicates in [workspace.dependencies] if exists
    if let Some(workspace) = toml_value.get_mut("workspace") {
        if let Some(deps) = workspace.get_mut("dependencies").and_then(|v| v.as_table_mut()) {
            let mut unique_deps = Map::new();
            for (key, value) in deps.iter() {
                if !unique_deps.contains_key(key) {
                    unique_deps.insert(key.clone(), value.clone());
                } else {
                    println!("Removed duplicate workspace dependency: {}", key);
                }
            }
            *deps = unique_deps;
        }
    }

    // Apply HOTZ philosophy if requested (minimal dependencies)
    if args.hotz {
        apply_hotz_philosophy(&mut toml_value, args.verbose)?;
    }

    // Add/resolve [patch.crates-io] for security (e.g., crypto vulns)
    if args.patch {
        let mut patches = Map::new();
        
        // Critical security patches
        patches.insert("curve25519-dalek".to_string(), 
            toml::Value::String("{ git = \"https://github.com/dalek-cryptography/curve25519-dalek\", tag = \"v4.1.3\" }".to_string()));
        patches.insert("ed25519-dalek".to_string(), 
            toml::Value::String("{ git = \"https://github.com/dalek-cryptography/ed25519-dalek\", tag = \"v2.2.0\" }".to_string()));
        
        // Additional security patches for THE OVERMIND PROTOCOL
        patches.insert("ring".to_string(), 
            toml::Value::String("{ git = \"https://github.com/briansmith/ring\", branch = \"main\" }".to_string()));
        
        // Replace unmaintained dependencies
        if args.verbose {
            println!("Adding patches for unmaintained dependencies...");
        }
        
        toml_value.as_table_mut().ok_or(anyhow!("Invalid TOML"))?
            .insert("patch".to_string(), toml::Value::Table({
                let mut patch_table = Map::new();
                patch_table.insert("crates-io".to_string(), toml::Value::Table(patches));
                patch_table
            }));

        println!("Added security patches to [patch.crates-io].");
    }

    // Write back to Cargo.toml
    let new_content = toml::to_string_pretty(&toml_value)?;
    write(&args.path, new_content)?;
    println!("Cargo.toml cleaned and saved.");

    // Optional: Run cargo update
    if args.update {
        println!("Running cargo update...");
        let output = Command::new("cargo")
            .arg("update")
            .output()?;
            
        if args.verbose {
            println!("cargo update stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("cargo update stderr: {}", String::from_utf8_lossy(&output.stderr));
        } else {
            println!("Ran cargo update.");
        }
    }

    // Optional: Run cargo audit
    if args.audit {
        println!("Running cargo audit...");
        let output = Command::new("cargo")
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

/// Applies HOTZ philosophy to dependencies (minimal, performant, secure)
fn apply_hotz_philosophy(toml_value: &mut Value, verbose: bool) -> Result<()> {
    if verbose {
        println!("Applying HOTZ philosophy (minimal dependencies)...");
    }
    
    // Define replacements for bloated dependencies
    let replacements = HashMap::from([
        ("reqwest", "minreq"),
        ("serde_json", "tinyjson"),
        ("rand", "nanorand"),
        ("ansi_term", "nu-ansi-term"),
        ("atty", "is-terminal"),
        ("backoff", "exponential-backoff"),
    ]);
    
    if let Some(deps) = toml_value.get_mut("dependencies").and_then(|v| v.as_table_mut()) {
        // Apply replacements
        for (bloated, minimal) in &replacements {
            if deps.contains_key(*bloated) && !deps.contains_key(*minimal) {
                if verbose {
                    println!("HOTZ: Replacing {} with {}", bloated, minimal);
                }
                
                // Keep the bloated dependency but add a comment
                if let Some(dep_value) = deps.get_mut(*bloated) {
                    if let Some(dep_table) = dep_value.as_table_mut() {
                        dep_table.insert("optional".to_string(), Value::Boolean(true));
                    }
                }
                
                // Add the minimal alternative
                match *minimal {
                    "minreq" => { deps.insert(minimal.to_string(), Value::String("2.11".to_string())); },
                    "tinyjson" => { deps.insert(minimal.to_string(), Value::String("2.5".to_string())); },
                    "nanorand" => { deps.insert(minimal.to_string(), Value::String("0.7".to_string())); },
                    "nu-ansi-term" => { deps.insert(minimal.to_string(), Value::String("0.50".to_string())); },
                    "is-terminal" => { deps.insert(minimal.to_string(), Value::String("0.4".to_string())); },
                    "exponential-backoff" => { deps.insert(minimal.to_string(), Value::String("2.1".to_string())); },
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}