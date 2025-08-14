//! JSON report generator

use super::{ReportGenerator, ReportConfig, ReportMetadata, aggregator::ReportDataAggregator, utils};
use crate::scan_v3::data::ScanResult;
use anyhow::Result;
use serde_json::{json, Value};
use std::time::UNIX_EPOCH;

/// JSON report generator - machine-friendly format
pub struct JsonReportGenerator;

impl ReportGenerator for JsonReportGenerator {
    fn generate(
        &self,
        result: &ScanResult,
        config: &ReportConfig,
        metadata: &ReportMetadata,
    ) -> Result<String> {
        let aggregator = ReportDataAggregator::new(result, config);
        let performance_stats = aggregator.file_performance_stats();
        let matches_by_type = aggregator.matches_by_type();
        let matches_by_file = aggregator.matches_by_file();
        
        let report = json!({
            "metadata": {
                "generated_at": metadata.generated_at
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "guardy_version": metadata.guardy_version,
                "scan_duration_ms": metadata.scan_duration_ms,
                "contains_real_secrets": config.display_secrets,
                "redaction_style": format!("{:?}", config.redaction_style),
            },
            "summary": {
                "total_secrets": result.matches.len(),
                "unique_secret_types": matches_by_type.len(),
                "files_scanned": result.stats.files_scanned,
                "files_with_secrets": result.files_with_secrets().len(),
                "total_warnings": result.warnings.len(),
                "scan_throughput_mb_per_sec": result.stats.throughput_mb_per_sec(),
            },
            "performance": {
                "total_bytes_processed": result.stats.total_bytes_processed,
                "total_lines_processed": performance_stats.total_lines_processed,
                "total_file_scan_time_ms": performance_stats.total_file_scan_time_ms,
                "lines_per_second": performance_stats.lines_per_second(metadata.scan_duration_ms),
                "files_failed": result.stats.files_failed,
                "slowest_files": performance_stats.slowest_files.iter().take(10).map(|(path, time_ms)| {
                    json!({
                        "file": path.as_ref(),
                        "scan_time_ms": time_ms,
                        "formatted_time": utils::format_duration_ms(*time_ms)
                    })
                }).collect::<Vec<Value>>(),
                "largest_files": performance_stats.largest_files.iter().take(10).map(|(path, size)| {
                    json!({
                        "file": path.as_ref(),
                        "size_bytes": size,
                        "formatted_size": utils::format_file_size(*size)
                    })
                }).collect::<Vec<Value>>(),
                "files_with_matches": performance_stats.files_with_matches.iter().map(|(path, count)| {
                    json!({
                        "file": path.as_ref(),
                        "match_count": count
                    })
                }).collect::<Vec<Value>>(),
            },
            "secrets_by_type": matches_by_type.iter().map(|(secret_type, matches)| {
                json!({
                    "type": secret_type.as_ref(),
                    "count": matches.len(),
                    "matches": matches.iter().map(|secret| {
                        json!({
                            "file": secret.file_path(),
                            "line": secret.line_number(),
                            "column_start": secret.coordinate().column_start,
                            "column_end": secret.coordinate().column_end(),
                            "matched_text": utils::get_secret_display_value_for_format(secret, config, super::ReportFormat::Json),
                            "pattern_description": secret.pattern_description.as_ref(),
                        })
                    }).collect::<Vec<Value>>()
                })
            }).collect::<Vec<Value>>(),
            "files_with_secrets": matches_by_file.iter().map(|(file_path, matches)| {
                json!({
                    "file": file_path.as_ref(),
                    "secret_count": matches.len(),
                    "secrets": matches.iter().map(|secret| {
                        json!({
                            "type": secret.secret_type.as_ref(),
                            "line": secret.line_number(),
                            "column_start": secret.coordinate().column_start,
                            "column_end": secret.coordinate().column_end(),
                            "matched_text": utils::get_secret_display_value_for_format(secret, config, super::ReportFormat::Json),
                        })
                    }).collect::<Vec<Value>>()
                })
            }).collect::<Vec<Value>>(),
            "warnings": result.warnings.iter().map(|warning| {
                json!({
                    "message": warning
                })
            }).collect::<Vec<Value>>(),
            "file_details": if config.include_file_timing {
                result.file_results.iter()
                    .filter(|f| f.success)
                    .map(|file_result| {
                        json!({
                            "file": file_result.file_path.as_ref(),
                            "lines_processed": file_result.lines_processed,
                            "file_size_bytes": file_result.file_size,
                            "scan_time_ms": file_result.scan_time_ms,
                            "matches_found": file_result.matches.len(),
                            "has_matches": file_result.has_matches(),
                        })
                    }).collect::<Vec<Value>>()
            } else {
                Vec::new()
            }
        });
        
        Ok(serde_json::to_string_pretty(&report)?)
    }
    
}