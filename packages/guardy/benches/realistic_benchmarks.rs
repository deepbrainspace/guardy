use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Configuration for realistic benchmarks
struct BenchConfig {
    guardy_path: String,
    gitleaks_available: bool,
    lefthook_available: bool,
}

impl BenchConfig {
    fn new() -> Self {
        let guardy_path = format!(
            "{}/target/release/guardy",
            env!("CARGO_MANIFEST_DIR").replace("/packages/guardy", "")
        );

        let gitleaks_available = Command::new("gitleaks")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let lefthook_available = Command::new("lefthook")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        Self {
            guardy_path,
            gitleaks_available,
            lefthook_available,
        }
    }
}

/// Get test project paths
fn get_test_projects() -> Vec<String> {
    let bench_dir = format!("{}/benches", env!("CARGO_MANIFEST_DIR"));
    vec![
        format!("{}/test-data/small-project", bench_dir),
        format!("{}/test-data/medium-project", bench_dir),
        format!("{}/test-data/large-project", bench_dir),
    ]
    .into_iter()
    .filter(|path| Path::new(path).exists())
    .collect()
}

/// Benchmark Guardy scanning performance
fn bench_guardy_scanning(c: &mut Criterion) {
    let config = BenchConfig::new();
    let projects = get_test_projects();

    if projects.is_empty() {
        println!("⚠️ No test projects found. Run 'make bench-setup' first.");
        return;
    }

    println!("🔍 Benchmarking Guardy scanning performance...");

    let mut group = c.benchmark_group("guardy_scanning");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    for project_path in projects {
        let project_name = Path::new(&project_path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace("-project", "");

        group.bench_with_input(
            BenchmarkId::new("scan", &project_name),
            &project_path,
            |b, path| {
                b.iter(|| {
                    let output = Command::new(&config.guardy_path)
                        .args(["scan", ".", "--quiet"])
                        .current_dir(path)
                        .output()
                        .expect("Failed to run guardy scan");

                    black_box(output.status.success())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Gitleaks vs Guardy if available
fn bench_scanning_comparison(c: &mut Criterion) {
    let config = BenchConfig::new();

    if !config.gitleaks_available {
        println!("⚠️ Gitleaks not available. Install with 'make bench-prepare'.");
        return;
    }

    let projects = get_test_projects();
    if projects.is_empty() {
        println!("⚠️ No test projects found. Run 'make bench-setup' first.");
        return;
    }

    println!("⚔️  Benchmarking Guardy vs Gitleaks...");

    let mut group = c.benchmark_group("scanning_comparison");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    // Use medium project for comparison
    if let Some(medium_project) = projects.iter().find(|p| p.contains("medium")) {
        group.bench_with_input(
            BenchmarkId::new("guardy", "medium"),
            medium_project,
            |b, path| {
                b.iter(|| {
                    let output = Command::new(&config.guardy_path)
                        .args(["scan", ".", "--quiet"])
                        .current_dir(path)
                        .output()
                        .expect("Failed to run guardy");

                    black_box(output.status.success())
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("gitleaks", "medium"),
            medium_project,
            |b, path| {
                b.iter(|| {
                    let output = Command::new("gitleaks")
                        .args([
                            "detect",
                            "--source",
                            ".",
                            "--no-git",
                            "--quiet",
                            "--exit-code",
                            "0",
                        ])
                        .current_dir(path)
                        .output()
                        .expect("Failed to run gitleaks");

                    black_box(output.status.success())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel worker scaling
fn bench_parallel_scaling(c: &mut Criterion) {
    let config = BenchConfig::new();
    let projects = get_test_projects();

    if projects.is_empty() {
        println!("⚠️ No test projects found. Run 'make bench-setup' first.");
        return;
    }

    println!("⚡ Benchmarking parallel worker scaling...");

    let mut group = c.benchmark_group("parallel_scaling");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    // Use largest available project
    if let Some(test_project) = projects.last() {
        for workers in [1, 2, 4, 8] {
            group.bench_with_input(
                BenchmarkId::new("workers", workers),
                &(test_project.clone(), workers),
                |b, (path, worker_count)| {
                    b.iter(|| {
                        let output = Command::new(&config.guardy_path)
                            .args(["scan", ".", "--quiet"])
                            .env("GUARDY_MAX_WORKERS", worker_count.to_string())
                            .current_dir(path)
                            .output()
                            .expect("Failed to run guardy");

                        black_box(output.status.success())
                    });
                },
            );
        }
    }

    group.finish();
}

/// Print system and tool information
fn print_bench_info() {
    println!("🚀 Guardy Realistic Benchmarks");
    println!("=============================");

    let config = BenchConfig::new();

    println!("🔧 Tool Status:");
    println!("  Guardy:   ✅ {}", config.guardy_path);
    println!(
        "  Gitleaks: {}",
        if config.gitleaks_available {
            "✅ Available"
        } else {
            "❌ Not found"
        }
    );
    println!(
        "  Lefthook: {}",
        if config.lefthook_available {
            "✅ Available"
        } else {
            "❌ Not found"
        }
    );

    let projects = get_test_projects();
    println!("\n📊 Test Projects:");
    if projects.is_empty() {
        println!("  ❌ No test projects found!");
        println!("  💡 Run 'make bench-setup' to generate test data");
    } else {
        for project in &projects {
            if let Ok(entries) = std::fs::read_dir(project) {
                let file_count = entries.count();
                let name = Path::new(project).file_name().unwrap().to_string_lossy();
                println!("  📁 {name} ({file_count} files)");
            }
        }
    }

    println!("\n🎯 Benchmark Categories:");
    println!("  🔍 Scanning Performance - Guardy across different project sizes");
    if config.gitleaks_available {
        println!("  ⚔️  Tool Comparison - Guardy vs Gitleaks");
    }
    println!("  ⚡ Parallel Scaling - Worker thread performance");

    println!();
}

fn setup_criterion() -> Criterion {
    print_bench_info();

    Criterion::default().with_output_color(true).with_plots()
}

criterion_group!(
    name = benches;
    config = setup_criterion();
    targets = bench_guardy_scanning, bench_scanning_comparison, bench_parallel_scaling
);

criterion_main!(benches);
