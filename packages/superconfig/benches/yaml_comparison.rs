//! YAML-specific benchmarks comparing superconfig against raw serde_yaml_bw
//!
//! This benchmark suite focuses on YAML performance to understand:
//! - How YAML parsing compares to JSON parsing
//! - Fast-config YAML performance vs raw serde_yaml_bw
//! - Impact of YAML file size on parsing performance

use criterion::{criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};
use std::fs;
use std::hint::black_box;
use tempfile::TempDir;

// Same config structure as main benchmarks for consistency
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
struct BenchmarkConfig {
    pub app: AppSettings,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: FeatureFlags,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
struct AppSettings {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub debug: bool,
    pub workers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub ssl_mode: String,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub request_timeout: u64,
    pub max_request_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
struct FeatureFlags {
    pub auth: bool,
    pub metrics: bool,
    pub caching: bool,
    pub rate_limiting: bool,
    pub webhooks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub structured: bool,
    pub file_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub collection_interval: u64,
    pub export_format: String,
}

/// Create YAML versions of realistic configs for benchmarking
/// Returns: (basic_microservice_yaml, complex_webapp_yaml, enterprise_config_yaml)
fn create_realistic_yaml_configs() -> (String, String, String) {
    // Basic Microservice YAML
    let basic_config = BenchmarkConfig {
        app: AppSettings {
            name: "payment-service".to_string(),
            version: "1.2.3".to_string(),
            environment: "production".to_string(),
            debug: false,
            workers: 2,
        },
        database: DatabaseConfig {
            url: "postgres://localhost/payments".to_string(),
            max_connections: 5,
            timeout_seconds: 30,
            ssl_mode: "require".to_string(),
            retry_attempts: 3,
        },
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            tls_enabled: true,
            request_timeout: 30,
            max_request_size: 1048576,
        },
        features: FeatureFlags {
            auth: true,
            metrics: false,
            caching: false,
            rate_limiting: true,
            webhooks: false,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
            structured: true,
            file_output: None,
        },
        metrics: MetricsConfig {
            enabled: false,
            endpoint: "/health".to_string(),
            collection_interval: 60,
            export_format: "none".to_string(),
        },
    };

    // Complex Web Application YAML
    let complex_config = BenchmarkConfig {
        app: AppSettings {
            name: "social-platform-api".to_string(),
            version: "2.4.1".to_string(),
            environment: "production".to_string(),
            debug: false,
            workers: 8,
        },
        database: DatabaseConfig {
            url: "postgres://db-cluster.internal:5432/social_platform?sslmode=require&pool_max_conns=50".to_string(),
            max_connections: 50,
            timeout_seconds: 60,
            ssl_mode: "require".to_string(),
            retry_attempts: 5,
        },
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8443,
            tls_enabled: true,
            request_timeout: 120,
            max_request_size: 10485760,
        },
        features: FeatureFlags {
            auth: true,
            metrics: true,
            caching: true,
            rate_limiting: true,
            webhooks: true,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "structured-json".to_string(),
            structured: true,
            file_output: Some("/var/log/social-platform/app.log".to_string()),
        },
        metrics: MetricsConfig {
            enabled: true,
            endpoint: "/metrics/prometheus".to_string(),
            collection_interval: 30,
            export_format: "prometheus".to_string(),
        },
    };

    // Enterprise Application YAML
    let enterprise_config = BenchmarkConfig {
        app: AppSettings {
            name: "enterprise-commerce-platform".to_string(),
            version: "3.2.1-enterprise-release-candidate-2".to_string(),
            environment: "production-us-east-1-primary-cluster".to_string(),
            debug: false,
            workers: 64,
        },
        database: DatabaseConfig {
            url: "postgres://primary-db-cluster.internal.enterprise.corp.com:5432/commerce_platform_production_v3?sslmode=require&pool_max_conns=200&connect_timeout=30&application_name=commerce-api".to_string(),
            max_connections: 200,
            timeout_seconds: 180,
            ssl_mode: "require-with-certificate-validation".to_string(),
            retry_attempts: 8,
        },
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8443,
            tls_enabled: true,
            request_timeout: 300,
            max_request_size: 268435456,
        },
        features: FeatureFlags {
            auth: true,
            metrics: true,
            caching: true,
            rate_limiting: true,
            webhooks: true,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "structured-json-with-correlation-ids-and-distributed-tracing".to_string(),
            structured: true,
            file_output: Some("/var/log/enterprise-commerce-platform/application-with-audit-trail.log".to_string()),
        },
        metrics: MetricsConfig {
            enabled: true,
            endpoint: "/internal/observability/metrics/prometheus/detailed-with-custom-labels".to_string(),
            collection_interval: 10,
            export_format: "prometheus-with-histogram-buckets-and-business-metrics".to_string(),
        },
    };

    // Convert to YAML strings
    let basic_yaml = serde_yaml_bw::to_string(&basic_config).unwrap();
    let complex_yaml = serde_yaml_bw::to_string(&complex_config).unwrap();
    let enterprise_yaml = serde_yaml_bw::to_string(&enterprise_config).unwrap();

    // Calculate and log actual sizes
    println!("YAML Config sizes:");
    println!("  Basic Microservice: {} bytes", basic_yaml.len());
    println!("  Complex Web App: {} bytes", complex_yaml.len());
    println!("  Enterprise Config: {} bytes", enterprise_yaml.len());

    (basic_yaml, complex_yaml, enterprise_yaml)
}

/// Benchmark superconfig YAML performance (without caching for fair comparison)
fn bench_superconfig_yaml(c: &mut Criterion, config_content: &str, size_name: &str) {
    let temp_dir = TempDir::new().unwrap();
    // Use unique config names to prevent LazyLock reuse across scenarios
    let config_name = format!("benchmark_{}", size_name);
    let config_file = temp_dir.path().join(format!("{}.yaml", config_name));
    fs::write(&config_file, config_content).unwrap();
    
    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let mut group = c.benchmark_group(format!("superconfig_yaml_{}", size_name));
    
    // Test YAML load without caching - use unique config name each time
    group.bench_function("load", |b| {
        b.iter(|| {
            let config = superconfig::FastConfig::<BenchmarkConfig>::load(&config_name).unwrap();
            black_box(config.clone_config())
        })
    });
    
    // Static access test - simulate LazyLock behavior
    group.bench_function("static_simulation", |b| {
        // Pre-load once to simulate LazyLock initialization
        let preloaded = superconfig::FastConfig::<BenchmarkConfig>::load(&config_name).unwrap();
        let preloaded_config = preloaded.clone_config();
        
        b.iter(|| {
            // This simulates repeated access after LazyLock initialization
            black_box(&preloaded_config)
        })
    });
    
    group.finish();
    std::env::set_current_dir(original_dir).unwrap();
}

/// Benchmark raw serde_yaml_bw with file I/O (fair comparison)
fn bench_raw_serde_yaml(c: &mut Criterion, config_content: &str, size_name: &str) {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("benchmark.yaml");
    fs::write(&config_file, config_content).unwrap();
    
    let mut group = c.benchmark_group(format!("raw_serde_yaml_{}", size_name));
    
    group.bench_function("load", |b| {
        b.iter(|| {
            // Read file + parse YAML (same as superconfig does)
            let file_content = fs::read_to_string(&config_file).unwrap();
            let config: BenchmarkConfig = serde_yaml_bw::from_str(&file_content).unwrap();
            black_box(config)
        })
    });
    
    group.finish();
}

/// Benchmark config-rs YAML performance
fn bench_config_rs_yaml(c: &mut Criterion, config_content: &str, size_name: &str) {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("benchmark.yaml");
    fs::write(&config_file, config_content).unwrap();
    
    let mut group = c.benchmark_group(format!("config_rs_yaml_{}", size_name));
    
    group.bench_function("load", |b| {
        b.iter(|| {
            let settings = config::Config::builder()
                .add_source(config::File::with_name(config_file.to_str().unwrap()))
                .build()
                .unwrap();
            let config: BenchmarkConfig = settings.try_deserialize().unwrap();
            black_box(config)
        })
    });
    
    group.finish();
}

/// Benchmark figment YAML performance
fn bench_figment_yaml(c: &mut Criterion, config_content: &str, size_name: &str) {
    use figment::providers::{Format, Yaml};
    
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("benchmark.yaml");
    fs::write(&config_file, config_content).unwrap();
    
    let mut group = c.benchmark_group(format!("figment_yaml_{}", size_name));
    
    group.bench_function("load", |b| {
        b.iter(|| {
            let figment = figment::Figment::new()
                .merge(Yaml::file(config_file.clone()));
            let config: BenchmarkConfig = figment.extract().unwrap();
            black_box(config)
        })
    });
    
    group.finish();
}

/// YAML parsing only benchmark (no file I/O)
fn bench_yaml_parsing_only(c: &mut Criterion, config_content: &str, size_name: &str) {
    let mut group = c.benchmark_group(format!("yaml_parsing_only_{}", size_name));
    
    group.bench_function("serde_yaml_parse", |b| {
        b.iter(|| {
            // Direct YAML parsing (no file I/O)
            let config: BenchmarkConfig = serde_yaml_bw::from_str(config_content).unwrap();
            black_box(config)
        })
    });
    
    group.finish();
}

/// Main YAML benchmark function
fn yaml_benchmarks(c: &mut Criterion) {
    let (basic_yaml, complex_yaml, enterprise_yaml) = create_realistic_yaml_configs();
    
    // Benchmark all libraries with YAML configs
    let configs = [
        ("basic_microservice", &basic_yaml),
        ("complex_webapp", &complex_yaml),
        ("enterprise_config", &enterprise_yaml),
    ];
    
    for (scenario_name, config_content) in configs.iter() {
        // Benchmark each approach across realistic YAML scenarios
        bench_superconfig_yaml(c, config_content, scenario_name);
        bench_raw_serde_yaml(c, config_content, scenario_name);
        bench_config_rs_yaml(c, config_content, scenario_name);
        bench_figment_yaml(c, config_content, scenario_name);
        bench_yaml_parsing_only(c, config_content, scenario_name);
    }
}

criterion_group!(benches, yaml_benchmarks);
criterion_main!(benches);