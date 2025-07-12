use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
};
use anyhow::{Result, anyhow};
use chrono::Local;
use clap::{Parser, Subcommand};
use toml_edit::{DocumentMut, Item, Table, value};
use serde_json::json;

// Import functions from the cargo_resolver module
use overmind_protocol::cargo_resolver::*;

const BACKUP_PREFIX: &str = "Cargo.toml.backup_";

#[derive(Parser)]
#[clap(
    name = "cargo-dependency-resolver",
    version = "1.1",
    about = "Advanced Rust dependency conflict resolver with security & minimalism enhancements",
    author = "THE OVERMIND PROTOCOL v4.1"
)]
struct Cli {
    #[clap(default_value = ".")]
    path: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clean duplicates and resolve conflicts
    Clean {
        #[clap(short, long)]
        verbose: bool,
        
        #[clap(short, long)]
        update: bool,
        
        #[clap(short, long)]
        audit: bool,
        
        #[clap(short, long)]
        patch: bool,
        
        #[clap(long)]
        hotz: bool,
    },
    
    /// Restore from backup
    Restore {
        /// Backup timestamp (YYYYMMDD_HHMMSS) or "latest"
        #[clap(default_value = "latest")]
        timestamp: String,
    },
    
    /// Enhance security (patches + insecure crate replacement)
    Enhance {
        #[clap(short, long)]
        aggressive: bool,
        
        #[clap(short, long)]
        verbose: bool,
        
        #[clap(long)]
        sbom: bool,  // Generate SBOM report
    },
}

#[derive(Debug)]
struct DepInfo {
    version: String,
    source: Option<String>,
    required_by: HashSet<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Clean { verbose, update, audit, patch, hotz } => {
            clean_dependencies(&cli.path, verbose, update, audit, patch, hotz)?;
        },
        Commands::Restore { timestamp } => {
            restore_from_backup(&cli.path, &timestamp)?;
        },
        Commands::Enhance { aggressive, verbose, sbom } => {
            enhance_security(&cli.path, aggressive, verbose, sbom)?;
        },
    }

    Ok(())
}

fn clean_dependencies(
    project_path: &str,
    verbose: bool,
    run_update: bool,
    run_audit: bool,
    apply_patch: bool,
    apply_hotz: bool,
) -> Result<()> {
    let cargo_path = Path::new(project_path).join("Cargo.toml");
    let mut doc = read_and_backup_cargo(&cargo_path)?;
    
    let sections = vec!["dependencies", "dev-dependencies", "build-dependencies"];
    
    for section in sections {
        if let Some(deps) = doc.get_mut(section).and_then(Item::as_table_mut) {
            clean_section_duplicates(deps, verbose, section);
        }
        
        if let Some(workspace) = doc.get_mut("workspace") {
            if let Some(ws_deps) = workspace.get_mut(section).and_then(Item::as_table_mut) {
                clean_section_duplicates(ws_deps, verbose, &format!("workspace.{}", section));
            }
        }
    }
    
    if apply_hotz {
        apply_hotz_philosophy(&mut doc, verbose)?;
    }
    
    if apply_patch {
        apply_security_patches(&mut doc, verbose)?;
    }
    
    write_cargo_toml(&cargo_path, doc.to_string())?;
    
    if run_update {
        run_cargo_command(project_path, "update")?;
    }
    
    if run_audit {
        run_cargo_audit(project_path)?;
    }
    
    Ok(())
}



fn enhance_security(project_path: &str, aggressive: bool, verbose: bool, generate_sbom: bool) -> Result<()> {
    let cargo_path = Path::new(project_path).join("Cargo.toml");
    let mut doc = read_and_backup_cargo(&cargo_path)?;
    
    apply_security_patches(&mut doc, verbose)?;
    
    let insecure_crates = vec!["chrono", "time@0.1", "openssl-sys@0.9"];
    remove_insecure_crates(&mut doc, &insecure_crates, aggressive, verbose)?;
    
    write_cargo_toml(&cargo_path, doc.to_string())?;
    
    if generate_sbom {
        generate_sbom_report(project_path)?;
    }
    
    Ok(())
}

















fn run_cargo_audit(project_path: &str) -> Result<()> {
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

fn run_cargo_command(project_path: &str, command: &str) -> Result<()> {
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


