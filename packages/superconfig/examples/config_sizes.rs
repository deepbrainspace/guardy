use serde::{Deserialize, Serialize};

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

fn main() {
    // Basic Microservice
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
    }).unwrap();

    println!("Basic Microservice: {} bytes", basic_microservice.len());
    println!("JSON content:\n{}", basic_microservice);
}