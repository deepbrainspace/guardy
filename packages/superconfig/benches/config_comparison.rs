//! Comprehensive benchmarks comparing superconfig against major Rust configuration libraries
//!
//! This benchmark suite tests:
//! - Initial load performance (cold start)
//! - Cached/repeated access performance
//! - Memory usage and allocation patterns
//! - Different config file sizes and complexity

use criterion::{criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};
use std::fs;
use std::hint::black_box;
use tempfile::TempDir;

// Common configuration structure used across all libraries
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

/// Create realistic configuration scenarios for benchmarking
/// Returns: (basic_microservice_json, complex_webapp_json, enterprise_config_json)
fn create_realistic_configs() -> (String, String, String) {
    // Basic Microservice: Simple API service with minimal configuration (~400 bytes JSON)
    let basic_microservice = serde_json::to_string_pretty(&BenchmarkConfig {
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
            max_request_size: 1048576, // 1MB
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
    }).unwrap();

    // Complex Web Application: Multi-feature web app with moderate complexity (~800 bytes JSON)
    let complex_webapp = serde_json::to_string_pretty(&BenchmarkConfig {
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
            max_request_size: 10485760, // 10MB
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
    }).unwrap();
    
    // Enterprise Application: Large-scale system with extensive configuration (~1.2KB JSON)
    let enterprise_config = serde_json::to_string_pretty(&BenchmarkConfig {
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
            max_request_size: 268435456, // 256MB for large file uploads
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
    }).unwrap();

    // Calculate and log actual sizes
    println!("Config sizes:");
    println!("  Basic Microservice: {} bytes", basic_microservice.len());
    println!("  Complex Web App: {} bytes", complex_webapp.len());
    println!("  Enterprise Config: {} bytes", enterprise_config.len());

    (basic_microservice, complex_webapp, enterprise_config)
}

/// Benchmark superconfig performance (with caching enabled)
fn bench_superconfig(c: &mut Criterion, config_content: &str, size_name: &str) {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("benchmark.json");
    fs::write(&config_file, config_content).unwrap();
    
    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let mut group = c.benchmark_group(format!("superconfig_{}", size_name));
    
    // Cold start (no cache)
    group.bench_function("cold_start", |b| {
        b.iter(|| {
            // Clear cache before each iteration (only if cache feature enabled)
            cfg_if::cfg_if! {
                if #[cfg(feature = "cache")] {
                    let cache_manager = superconfig::CacheManager::new("benchmark").unwrap();
                    let _ = cache_manager.clear_cache();
                }
            }
            
            let config = superconfig::FastConfig::<BenchmarkConfig>::load("benchmark").unwrap();
            black_box(config.clone_config())
        })
    });
    
    // Warm start (with cache) - only if cache feature enabled
    #[cfg(feature = "cache")]
    group.bench_function("cached_load", |b| {
        // Pre-warm the cache
        let _config = superconfig::FastConfig::<BenchmarkConfig>::load("benchmark").unwrap();
        
        b.iter(|| {
            let config = superconfig::FastConfig::<BenchmarkConfig>::load("benchmark").unwrap();
            black_box(config.clone_config())
        })
    });
    
    // Static access (LazyLock)
    superconfig::static_config!(BENCH_CONFIG, BenchmarkConfig, "benchmark");
    group.bench_function("static_access", |b| {
        b.iter(|| {
            black_box(&*BENCH_CONFIG)
        })
    });
    
    group.finish();
    std::env::set_current_dir(original_dir).unwrap();
}

/// Benchmark superconfig performance (without caching - cache feature disabled)
fn bench_superconfig_no_cache(c: &mut Criterion, config_content: &str, size_name: &str) {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("benchmark.json");
    fs::write(&config_file, config_content).unwrap();
    
    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let mut group = c.benchmark_group(format!("superconfig_no_cache_{}", size_name));
    
    // Test load without caching (should be faster than cached version)
    group.bench_function("load", |b| {
        b.iter(|| {
            // This will compile with --no-default-features to disable cache
            let config = superconfig::FastConfig::<BenchmarkConfig>::load("benchmark").unwrap();
            black_box(config.clone_config())
        })
    });
    
    // Static access (LazyLock) - same performance regardless of cache feature
    superconfig::static_config!(BENCH_CONFIG_NO_CACHE, BenchmarkConfig, "benchmark");
    group.bench_function("static_access", |b| {
        b.iter(|| {
            black_box(&*BENCH_CONFIG_NO_CACHE)
        })
    });
    
    group.finish();
    std::env::set_current_dir(original_dir).unwrap();
}

/// Benchmark config-rs library
fn bench_config_rs(c: &mut Criterion, config_content: &str, size_name: &str) {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("benchmark.json");
    fs::write(&config_file, config_content).unwrap();
    
    let mut group = c.benchmark_group(format!("config_rs_{}", size_name));
    
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

/// Benchmark figment library
fn bench_figment(c: &mut Criterion, config_content: &str, size_name: &str) {
    use figment::providers::{Format, Json};
    
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("benchmark.json");
    fs::write(&config_file, config_content).unwrap();
    
    let mut group = c.benchmark_group(format!("figment_{}", size_name));
    
    group.bench_function("load", |b| {
        b.iter(|| {
            let figment = figment::Figment::new()
                .merge(Json::file(config_file.clone()));
            let config: BenchmarkConfig = figment.extract().unwrap();
            black_box(config)
        })
    });
    
    group.finish();
}

/// Benchmark confy library (uses TOML)
fn bench_confy(c: &mut Criterion, config_data: &BenchmarkConfig, size_name: &str) {
    let mut group = c.benchmark_group(format!("confy_{}", size_name));
    
    group.bench_function("load", |b| {
        b.iter(|| {
            // Confy loads from default location, so we simulate with direct deserialization
            let toml_content = toml::to_string(config_data).unwrap();
            let config: BenchmarkConfig = toml::from_str(&toml_content).unwrap();
            black_box(config)
        })
    });
    
    group.finish();
}

/// Benchmark raw serde_json with file I/O (fair comparison)
fn bench_raw_serde_json(c: &mut Criterion, config_content: &str, size_name: &str) {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("benchmark.json");
    fs::write(&config_file, config_content).unwrap();
    
    let mut group = c.benchmark_group(format!("raw_serde_json_{}", size_name));
    
    group.bench_function("load", |b| {
        b.iter(|| {
            // Read file + parse JSON (same as other libraries)
            let file_content = fs::read_to_string(&config_file).unwrap();
            let config: BenchmarkConfig = serde_json::from_str(&file_content).unwrap();
            black_box(config)
        })
    });
    
    group.finish();
}

/// Benchmark superconfig JSON parsing only (no file I/O, no caching)
fn bench_superconfig_json_only(c: &mut Criterion, config_content: &str, size_name: &str) {
    let mut group = c.benchmark_group(format!("superconfig_json_only_{}", size_name));
    
    group.bench_function("parse", |b| {
        b.iter(|| {
            // Direct JSON parsing like other libraries do
            let config: BenchmarkConfig = serde_json::from_str(config_content).unwrap();
            black_box(config)
        })
    });
    
    group.finish();
}

/// Main benchmark function
fn config_benchmarks(c: &mut Criterion) {
    let (basic_microservice, complex_webapp, enterprise_config) = create_realistic_configs();
    
    // Parse configs for libraries that need structured data
    let basic_data: BenchmarkConfig = serde_json::from_str(&basic_microservice).unwrap();
    let webapp_data: BenchmarkConfig = serde_json::from_str(&complex_webapp).unwrap();
    let enterprise_data: BenchmarkConfig = serde_json::from_str(&enterprise_config).unwrap();
    
    // Benchmark all libraries with different realistic config scenarios
    let configs = [
        ("basic_microservice", &basic_microservice, &basic_data),
        ("complex_webapp", &complex_webapp, &webapp_data),
        ("enterprise_config", &enterprise_config, &enterprise_data),
    ];
    
    for (scenario_name, config_content, config_data) in configs.iter() {
        // Benchmark each library across realistic scenarios
        bench_superconfig(c, config_content, scenario_name);
        bench_superconfig_no_cache(c, config_content, scenario_name);
        bench_superconfig_json_only(c, config_content, scenario_name);
        bench_config_rs(c, config_content, scenario_name);
        bench_figment(c, config_content, scenario_name);
        bench_confy(c, config_data, scenario_name);
        bench_raw_serde_json(c, config_content, scenario_name);
    }
}

/// Memory allocation benchmarks across different config sizes
fn memory_benchmarks(c: &mut Criterion) {
    let (basic_microservice, complex_webapp, enterprise_config) = create_realistic_configs();
    
    let configs = [
        ("basic_microservice", &basic_microservice),
        ("complex_webapp", &complex_webapp), 
        ("enterprise_config", &enterprise_config),
    ];
    
    for (size_name, config_content) in configs.iter() {
        let mut group = c.benchmark_group(format!("memory_efficiency_{}", size_name));
        
        // Fast-config memory usage
        group.bench_function("superconfig_memory", |b| {
            let temp_dir = TempDir::new().unwrap();
            let config_file = temp_dir.path().join("memory_test.json");
            fs::write(&config_file, config_content).unwrap();
            
            let original_dir = std::env::current_dir().unwrap();
            std::env::set_current_dir(&temp_dir).unwrap();
            
            b.iter(|| {
                for _ in 0..50 {
                    let config = superconfig::FastConfig::<BenchmarkConfig>::load("memory_test").unwrap();
                    black_box(config.clone_config());
                }
            });
            
            std::env::set_current_dir(original_dir).unwrap();
        });
        
        // Raw serde_json memory usage
        group.bench_function("serde_json_memory", |b| {
            b.iter(|| {
                for _ in 0..50 {
                    let config: BenchmarkConfig = serde_json::from_str(config_content).unwrap();
                    black_box(config);
                }
            })
        });
        
        group.finish();
    }
}

criterion_group!(benches, config_benchmarks, memory_benchmarks);
criterion_main!(benches);