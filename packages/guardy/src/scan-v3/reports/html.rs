//! HTML report generator - modern, interactive reports

use super::{ReportGenerator, ReportConfig, ReportMetadata, aggregator::ReportDataAggregator, utils};
use crate::scan_v3::data::ScanResult;
use anyhow::Result;
use std::time::UNIX_EPOCH;

/// HTML report generator - human-friendly, interactive format
pub struct HtmlReportGenerator;

impl ReportGenerator for HtmlReportGenerator {
    fn generate(
        &self,
        result: &ScanResult,
        config: &ReportConfig,
        metadata: &ReportMetadata,
    ) -> Result<String> {
        let aggregator = ReportDataAggregator::new(result, config);
        let performance_stats = aggregator.file_performance_stats();
        let matches_by_type = aggregator.matches_by_type();
        
        let timestamp = metadata.generated_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let safety_banner = if config.display_secrets {
            r#"<div class="safety-banner danger">🚨 CONTAINS REAL SECRETS - Handle with care!</div>"#
        } else {
            r#"<div class="safety-banner safe">🛡️ Secrets Redacted - Safe for sharing</div>"#
        };
        
        let secrets_section = Self::generate_secrets_section(&matches_by_type, config);
        let performance_section = if config.include_file_timing {
            Self::generate_performance_section(&performance_stats, metadata.scan_duration_ms)
        } else {
            String::new()
        };
        let warnings_section = Self::generate_warnings_section(&result.warnings);
        
        let html = format!(
            include_str!("templates/report.html"),
            safety_banner = safety_banner,
            timestamp = timestamp,
            total_secrets = result.matches.len(),
            files_scanned = result.stats.files_scanned,
            files_with_secrets = result.files_with_secrets().len(),
            total_warnings = result.warnings.len(),
            scan_time = utils::format_duration_ms(metadata.scan_duration_ms),
            throughput = format!("{:.1} MB/s", result.stats.throughput_mb_per_sec()),
            secrets_section = secrets_section,
            performance_section = performance_section,
            warnings_section = warnings_section,
            guardy_version = metadata.guardy_version,
        );
        
        Ok(html)
    }
    
}

impl HtmlReportGenerator {
    fn generate_secrets_section(
        matches_by_type: &[(std::sync::Arc<str>, Vec<&crate::scan_v3::data::SecretMatch>)],
        config: &ReportConfig,
    ) -> String {
        if matches_by_type.is_empty() {
            return r#"<div class="no-secrets">✅ No secrets found!</div>"#.to_string();
        }
        
        let mut sections = String::new();
        
        for (secret_type, secrets) in matches_by_type {
            let section_id = format!("secrets-{}", secret_type.replace(' ', "-").to_lowercase());
            
            sections.push_str(&format!(
                r#"
        <div class="section">
            <div class="section-header" onclick="toggleSection('{}')">
                <span>🔑 {} ({} matches)</span>
                <button class="toggle-btn">▼</button>
            </div>
            <div id="{}" class="section-content collapsible">
                <input type="text" class="search-box" placeholder="Filter {} results..."
                       onkeyup="filterTable('search-{}', 'table-{}')" id="search-{}">
                <table id="table-{}">
                    <thead>
                        <tr>
                            <th>File</th>
                            <th>Line</th>
                            <th>Position</th>
                            <th>Secret</th>
                            <th>Pattern</th>
                        </tr>
                    </thead>
                    <tbody>
"#,
                section_id, secret_type, secrets.len(), section_id,
                secret_type, section_id, section_id, section_id, section_id
            ));
            
            for secret in secrets {
                let file_path = utils::html_escape(secret.file_path());
                let secret_value = utils::html_escape(&utils::get_secret_display_value_for_format(secret, config, super::ReportFormat::Html));
                let pattern_desc = utils::html_escape(secret.pattern_description.as_ref());
                
                sections.push_str(&format!(
                    r#"
                        <tr>
                            <td class="file-path">{}</td>
                            <td>{}</td>
                            <td>{}:{}</td>
                            <td class="secret-value">{}</td>
                            <td class="pattern-desc">{}</td>
                        </tr>
"#,
                    file_path,
                    secret.line_number(),
                    secret.coordinate().column_start,
                    secret.coordinate().column_end(),
                    secret_value,
                    pattern_desc
                ));
            }
            
            sections.push_str(
                r#"
                    </tbody>
                </table>
            </div>
        </div>
"#,
            );
        }
        
        sections
    }
    
    fn generate_performance_section(
        performance_stats: &super::aggregator::FilePerformanceStats,
        scan_duration_ms: u64,
    ) -> String {
        if performance_stats.slowest_files.is_empty() && performance_stats.largest_files.is_empty() {
            return String::new();
        }
        
        let mut section = String::from(r#"
        <div class="section">
            <div class="section-header" onclick="toggleSection('performance')">
                <span>⚡ Performance Analysis</span>
                <button class="toggle-btn">▼</button>
            </div>
            <div id="performance" class="section-content collapsible">
                <div class="performance-grid">
                    <div class="perf-card">
                        <h4>Scan Performance</h4>
                        <div class="perf-stat">
                            <span class="perf-label">Lines/second:</span>
                            <span class="perf-value">{:.0}</span>
                        </div>
                        <div class="perf-stat">
                            <span class="perf-label">Total lines:</span>
                            <span class="perf-value">{total_lines}</span>
                        </div>
                        <div class="perf-stat">
                            <span class="perf-label">File scan time:</span>
                            <span class="perf-value">{scan_time} ms</span>
                        </div>
                    </div>
"#);
        
        section = section.replace("{:.0}", &format!("{:.0}", performance_stats.lines_per_second(scan_duration_ms)));
        section = section.replace("{total_lines}", &performance_stats.total_lines_processed.to_string());
        section = section.replace("{scan_time}", &performance_stats.total_file_scan_time_ms.to_string());
        
        // Slowest files
        if !performance_stats.slowest_files.is_empty() {
            section.push_str(r#"
                    <div class="perf-card">
                        <h4>Slowest Files</h4>
"#);
            for (file_path, time_ms) in performance_stats.slowest_files.iter().take(5) {
                section.push_str(&format!(
                    r#"
                        <div class="perf-item">
                            <span class="file-name">{}</span>
                            <span class="perf-time">{}</span>
                        </div>
"#,
                    utils::html_escape(file_path.as_ref()),
                    utils::format_duration_ms(*time_ms)
                ));
            }
            section.push_str("                    </div>\n");
        }
        
        // Largest files
        if !performance_stats.largest_files.is_empty() {
            section.push_str(r#"
                    <div class="perf-card">
                        <h4>Largest Files</h4>
"#);
            for (file_path, size_bytes) in performance_stats.largest_files.iter().take(5) {
                section.push_str(&format!(
                    r#"
                        <div class="perf-item">
                            <span class="file-name">{}</span>
                            <span class="perf-size">{}</span>
                        </div>
"#,
                    utils::html_escape(file_path.as_ref()),
                    utils::format_file_size(*size_bytes)
                ));
            }
            section.push_str("                    </div>\n");
        }
        
        // Files with matches
        if !performance_stats.files_with_matches.is_empty() {
            section.push_str(r#"
                    <div class="perf-card">
                        <h4>Files with Matches</h4>
"#);
            for (file_path, match_count) in performance_stats.files_with_matches.iter().take(10) {
                section.push_str(&format!(
                    r#"
                        <div class="perf-item">
                            <span class="file-name">{}</span>
                            <span class="match-count">{} secrets</span>
                        </div>
"#,
                    utils::html_escape(file_path.as_ref()),
                    match_count
                ));
            }
            section.push_str("                    </div>\n");
        }
        
        section.push_str(r#"
                </div>
            </div>
        </div>
"#);
        
        section
    }
    
    fn generate_warnings_section(warnings: &[String]) -> String {
        if warnings.is_empty() {
            return String::new();
        }
        
        let mut section = String::from(r#"
        <div class="section">
            <div class="section-header" onclick="toggleSection('warnings')">
                <span>⚠️ Warnings ({} issues)</span>
                <button class="toggle-btn">▼</button>
            </div>
            <div id="warnings" class="section-content collapsible">
                <table>
                    <thead>
                        <tr>
                            <th>Warning Message</th>
                        </tr>
                    </thead>
                    <tbody>
"#);
        
        section = section.replace("{}", &warnings.len().to_string());
        
        for warning in warnings {
            section.push_str(&format!(
                r#"
                        <tr>
                            <td class="warning-msg">{}</td>
                        </tr>
"#,
                utils::html_escape(warning)
            ));
        }
        
        section.push_str(r#"
                    </tbody>
                </table>
            </div>
        </div>
"#);
        
        section
    }
}