use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use tempfile::TempDir;
use toml_edit::{DocumentMut, Item, Table, value};

fn create_cargo_toml_with_deps(num_deps: usize, duplicate_ratio: f32) -> String {
    let mut content = String::from(r#"
[package]
name = "benchmark-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#);
    
    for i in 0..num_deps {
        content.push_str(&format!("dep{} = \"1.0\"\n", i));
        
        // Add duplicates based on ratio
        if (i as f32 / num_deps as f32) < duplicate_ratio {
            content.push_str(&format!("dep{} = \"1.1\"\n", i));
        }
    }
    
    content.push_str("\n[dev-dependencies]\n");
    for i in 0..(num_deps / 4) {
        content.push_str(&format!("dev-dep{} = \"1.0\"\n", i));
    }
    
    content.push_str("\n[build-dependencies]\n");
    for i in 0..(num_deps / 8) {
        content.push_str(&format!("build-dep{} = \"1.0\"\n", i));
    }
    
    content
}

fn benchmark_duplicate_removal(c: &mut Criterion) {
    let mut group = c.benchmark_group("duplicate_removal");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("deps", size),
            size,
            |b, &size| {
                let content = create_cargo_toml_with_deps(size, 0.2); // 20% duplicates
                
                b.iter(|| {
                    let mut doc: DocumentMut = black_box(content.parse().unwrap());
                    
                    // Duplicate removal logic
                    let sections = vec!["dependencies", "dev-dependencies", "build-dependencies"];
                    
                    for section in sections {
                        if let Some(deps) = doc.get_mut(section).and_then(Item::as_table_mut) {
                            let mut unique = Table::new();
                            for (name, item) in deps.iter() {
                                if !unique.contains_key(name) {
                                    unique.insert(name, item.clone());
                                }
                            }
                            *deps = unique;
                        }
                    }
                    
                    black_box(doc)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_hotz_philosophy(c: &mut Criterion) {
    let mut group = c.benchmark_group("hotz_philosophy");
    
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("replacements", size),
            size,
            |b, &size| {
                let mut content = create_cargo_toml_with_deps(size, 0.0);
                // Add specific dependencies that will be replaced
                content.push_str("reqwest = \"0.11\"\n");
                content.push_str("serde_json = \"1.0\"\n");
                content.push_str("rand = \"0.8\"\n");
                content.push_str("chrono = \"0.4\"\n");
                
                b.iter(|| {
                    let mut doc: DocumentMut = black_box(content.parse().unwrap());
                    
                    // HOTZ philosophy replacements
                    let replacements = std::collections::HashMap::from([
                        ("reqwest", ("ureq", "0.6")),
                        ("serde_json", ("json", "0.12")),
                        ("rand", ("getrandom", "0.8")),
                        ("chrono", ("time", "0.3")),
                    ]);
                    
                    let sections = vec!["dependencies", "dev-dependencies", "build-dependencies"];
                    
                    for section in sections {
                        if let Some(deps) = doc.get_mut(section).and_then(Item::as_table_mut) {
                            for (bloated, (minimal, version)) in &replacements {
                                if deps.contains_key(*bloated) && !deps.contains_key(*minimal) {
                                    deps.remove(*bloated);
                                    deps.insert(minimal, value(version.to_string()));
                                }
                            }
                        }
                    }
                    
                    black_box(doc)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_security_patches(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_patches");
    
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("patches", size),
            size,
            |b, &size| {
                let content = create_cargo_toml_with_deps(size, 0.0);
                
                b.iter(|| {
                    let mut doc: DocumentMut = black_box(content.parse().unwrap());
                    
                    // Security patches application
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
                    
                    black_box(doc)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_toml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_parsing");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("parse", size),
            size,
            |b, &size| {
                let content = create_cargo_toml_with_deps(size, 0.1);
                
                b.iter(|| {
                    let doc: DocumentMut = black_box(content.parse().unwrap());
                    black_box(doc)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_toml_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_serialization");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            size,
            |b, &size| {
                let content = create_cargo_toml_with_deps(size, 0.1);
                let doc: DocumentMut = content.parse().unwrap();
                
                b.iter(|| {
                    let serialized = black_box(doc.to_string());
                    black_box(serialized)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");
    
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("read_write", size),
            size,
            |b, &size| {
                let content = create_cargo_toml_with_deps(size, 0.1);
                
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let cargo_path = temp_dir.path().join("Cargo.toml");
                    
                    // Write
                    fs::write(&cargo_path, &content).unwrap();
                    
                    // Read
                    let read_content = fs::read_to_string(&cargo_path).unwrap();
                    
                    // Parse
                    let doc: DocumentMut = read_content.parse().unwrap();
                    
                    // Serialize
                    let serialized = doc.to_string();
                    
                    // Write back
                    fs::write(&cargo_path, serialized).unwrap();
                    
                    black_box(temp_dir)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    for size in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("large_toml", size),
            size,
            |b, &size| {
                let content = create_cargo_toml_with_deps(size, 0.15);
                
                b.iter(|| {
                    let mut doc: DocumentMut = black_box(content.parse().unwrap());
                    
                    // Perform all operations
                    let sections = vec!["dependencies", "dev-dependencies", "build-dependencies"];
                    
                    // Duplicate removal
                    for section in &sections {
                        if let Some(deps) = doc.get_mut(section).and_then(Item::as_table_mut) {
                            let mut unique = Table::new();
                            for (name, item) in deps.iter() {
                                if !unique.contains_key(name) {
                                    unique.insert(name, item.clone());
                                }
                            }
                            *deps = unique;
                        }
                    }
                    
                    // HOTZ replacements
                    let replacements = std::collections::HashMap::from([
                        ("reqwest", ("ureq", "0.6")),
                        ("serde_json", ("json", "0.12")),
                    ]);
                    
                    for section in &sections {
                        if let Some(deps) = doc.get_mut(section).and_then(Item::as_table_mut) {
                            for (bloated, (minimal, version)) in &replacements {
                                if deps.contains_key(*bloated) && !deps.contains_key(*minimal) {
                                    deps.remove(*bloated);
                                    deps.insert(minimal, value(version.to_string()));
                                }
                            }
                        }
                    }
                    
                    // Security patches
                    let mut patches = Table::new();
                    patches.insert("curve25519-dalek", 
                        value("{ git = \"https://github.com/dalek-cryptography/curve25519-dalek\", tag = \"v4.1.3\" }"));
                    
                    let mut patch_table = Table::new();
                    patch_table.insert("crates-io", Item::Table(patches));
                    doc.as_table_mut().insert("patch", Item::Table(patch_table));
                    
                    black_box(doc)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_duplicate_removal,
    benchmark_hotz_philosophy,
    benchmark_security_patches,
    benchmark_toml_parsing,
    benchmark_toml_serialization,
    benchmark_file_operations,
    benchmark_memory_usage
);

criterion_main!(benches);
