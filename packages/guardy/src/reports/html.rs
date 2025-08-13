use crate::scan_v1::types::{SecretMatch, Warning};
use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct ReportGenerator;

#[derive(Debug, Clone)]
pub enum ReportFormat {
    Html,
    Json,
}

impl ReportGenerator {
    /// Generate a report in the specified format for large result sets
    pub fn generate_report(
        matches: &[&SecretMatch],
        warnings: &[&Warning],
        total_files: usize,
        total_skipped: usize,
        elapsed: Duration,
        output_dir: &Path,
        format: ReportFormat,
    ) -> Result<PathBuf> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let (report_filename, content) = match format {
            ReportFormat::Html => {
                let filename = format!("guardy-report-{timestamp}.html");
                let content = Self::generate_html_content(
                    matches,
                    warnings,
                    total_files,
                    total_skipped,
                    elapsed,
                )?;
                (filename, content)
            }
            ReportFormat::Json => {
                let filename = format!("guardy-report-{timestamp}.json");
                let content = Self::generate_json_content(
                    matches,
                    warnings,
                    total_files,
                    total_skipped,
                    elapsed,
                )?;
                (filename, content)
            }
        };

        let report_path = output_dir.join(&report_filename);
        fs::write(&report_path, content)?;
        Ok(report_path)
    }

    /// Generate JSON report (machine-friendly)
    fn generate_json_content(
        matches: &[&SecretMatch],
        warnings: &[&Warning],
        total_files: usize,
        total_skipped: usize,
        elapsed: Duration,
    ) -> Result<String> {
        let report = json!({
            "report_metadata": {
                "generated_at": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                "guardy_version": env!("CARGO_PKG_VERSION"),
                "scan_duration_ms": elapsed.as_millis(),
                "total_files_scanned": total_files,
                "total_files_skipped": total_skipped
            },
            "summary": {
                "total_secrets": matches.len(),
                "total_warnings": warnings.len()
            },
            "secrets": matches.iter().map(|s| json!({
                "file": s.file_path,
                "line": s.line_number,
                "type": s.secret_type,
                "content": s.line_content.trim(),
                "matched_text": s.matched_text,
                "start_pos": s.start_pos,
                "end_pos": s.end_pos,
                "pattern_description": s.pattern_description
            })).collect::<Vec<_>>(),
            "warnings": warnings.iter().map(|w| json!({
                "message": w.message
            })).collect::<Vec<_>>()
        });

        Ok(serde_json::to_string_pretty(&report)?)
    }

    /// Generate HTML report (human-friendly, interactive)
    fn generate_html_content(
        matches: &[&SecretMatch],
        warnings: &[&Warning],
        total_files: usize,
        total_skipped: usize,
        elapsed: Duration,
    ) -> Result<String> {
        let secrets_by_type = Self::group_secrets_by_type(matches);
        let warnings_by_type = Self::group_warnings_by_type(warnings);

        let secrets_section = Self::generate_secrets_html_section(&secrets_by_type);
        let warnings_section = Self::generate_warnings_html_section(&warnings_by_type);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Guardy Security Scan Report</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .header {{ border-bottom: 3px solid #e74c3c; padding-bottom: 20px; margin-bottom: 30px; }}
        .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin: 20px 0; }}
        .stat-card {{ background: #f8f9fa; padding: 15px; border-radius: 6px; text-align: center; }}
        .stat-number {{ font-size: 2em; font-weight: bold; color: #e74c3c; }}
        .section {{ margin: 30px 0; }}
        .section-header {{ background: #34495e; color: white; padding: 15px; cursor: pointer; user-select: none; border-radius: 4px 4px 0 0; }}
        .section-content {{ border: 1px solid #34495e; border-top: none; border-radius: 0 0 4px 4px; }}
        .collapsible {{ display: none; }}
        .collapsible.active {{ display: block; }}
        table {{ width: 100%; border-collapse: collapse; margin: 0; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background: #ecf0f1; font-weight: 600; }}
        .file-path {{ font-family: monospace; color: #2980b9; word-break: break-all; }}
        .warning-msg {{ color: #f39c12; font-family: monospace; }}
        .search-box {{ width: 100%; padding: 10px; margin: 10px 0; border: 1px solid #ddd; border-radius: 4px; }}
        .toggle-btn {{ float: right; background: none; border: none; color: white; font-size: 1.2em; }}
        tr:hover {{ background: #f8f9fa; }}
    </style>
    <script>
        function toggleSection(id) {{
            const content = document.getElementById(id);
            const btn = content.previousElementSibling.querySelector('.toggle-btn');
            if (content.classList.contains('active')) {{
                content.classList.remove('active');
                btn.textContent = '▼';
            }} else {{
                content.classList.add('active');
                btn.textContent = '▲';
            }}
        }}

        function filterTable(inputId, tableId) {{
            const input = document.getElementById(inputId);
            const table = document.getElementById(tableId);
            const filter = input.value.toLowerCase();
            const rows = table.getElementsByTagName('tr');

            for (let i = 1; i < rows.length; i++) {{
                const row = rows[i];
                const text = row.textContent.toLowerCase();
                row.style.display = text.includes(filter) ? '' : 'none';
            }}
        }}
    </script>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🛡️ Guardy Security Scan Report</h1>
            <p>Generated on {}</p>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-number">{}</div>
                <div>Secrets Found</div>
            </div>
            <div class="stat-card">
                <div class="stat-number">{}</div>
                <div>Files Scanned</div>
            </div>
            <div class="stat-card">
                <div class="stat-number">{}</div>
                <div>Files Skipped</div>
            </div>
            <div class="stat-card">
                <div class="stat-number">{}</div>
                <div>Warnings</div>
            </div>
            <div class="stat-card">
                <div class="stat-number">{:.1}s</div>
                <div>Scan Time</div>
            </div>
        </div>

        {}

        {}
    </div>
</body>
</html>"#,
            timestamp,
            matches.len(),
            total_files,
            total_skipped,
            warnings.len(),
            elapsed.as_secs_f64(),
            secrets_section,
            warnings_section
        );

        Ok(html)
    }

    fn group_secrets_by_type<'a>(
        matches: &'a [&'a SecretMatch],
    ) -> Vec<(String, Vec<&'a SecretMatch>)> {
        use std::collections::HashMap;

        let mut grouped: HashMap<String, Vec<&'a SecretMatch>> = HashMap::new();
        for secret in matches {
            grouped
                .entry(secret.secret_type.clone())
                .or_default()
                .push(*secret);
        }

        let mut result: Vec<_> = grouped.into_iter().collect();
        result.sort_by(|a, b| b.1.len().cmp(&a.1.len())); // Sort by count descending
        result
    }

    fn group_warnings_by_type<'a>(warnings: &'a [&'a Warning]) -> Vec<(String, Vec<&'a Warning>)> {
        use std::collections::HashMap;

        let mut grouped: HashMap<String, Vec<&'a Warning>> = HashMap::new();
        for warning in warnings {
            let warning_type = if warning.message.contains("Failed to scan") {
                "Scan Failures".to_string()
            } else if warning.message.contains("Walk error") {
                "Directory Walk Errors".to_string()
            } else {
                "Other Warnings".to_string()
            };

            grouped.entry(warning_type).or_default().push(*warning);
        }

        let mut result: Vec<_> = grouped.into_iter().collect();
        result.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        result
    }

    fn generate_secrets_html_section(secrets_by_type: &[(String, Vec<&SecretMatch>)]) -> String {
        let mut sections = String::new();

        for (secret_type, secrets) in secrets_by_type {
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
                            <th>Context</th>
                        </tr>
                    </thead>
                    <tbody>
"#,
                section_id,
                secret_type,
                secrets.len(),
                section_id,
                secret_type,
                section_id,
                section_id,
                section_id,
                section_id
            ));

            for secret in secrets {
                let file_path = secret
                    .file_path
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                let line_content = secret
                    .line_content
                    .trim()
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");

                sections.push_str(&format!(
                    r#"
                        <tr>
                            <td class="file-path">{}</td>
                            <td>{}</td>
                            <td>{}</td>
                        </tr>
"#,
                    file_path, secret.line_number, line_content
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

    fn generate_warnings_html_section(warnings_by_type: &[(String, Vec<&Warning>)]) -> String {
        if warnings_by_type.is_empty() {
            return String::new();
        }

        let mut sections = String::new();

        for (warning_type, warnings) in warnings_by_type {
            let section_id = format!("warnings-{}", warning_type.replace(' ', "-").to_lowercase());

            sections.push_str(&format!(
                r#"
        <div class="section">
            <div class="section-header" onclick="toggleSection('{}')">
                <span>⚠️ {} ({} warnings)</span>
                <button class="toggle-btn">▼</button>
            </div>
            <div id="{}" class="section-content collapsible">
                <input type="text" class="search-box" placeholder="Filter {} warnings..."
                       onkeyup="filterTable('search-{}', 'table-{}')" id="search-{}">
                <table id="table-{}">
                    <thead>
                        <tr>
                            <th>Warning Message</th>
                        </tr>
                    </thead>
                    <tbody>
"#,
                section_id,
                warning_type,
                warnings.len(),
                section_id,
                warning_type,
                section_id,
                section_id,
                section_id,
                section_id
            ));

            for warning in warnings {
                let message = warning
                    .message
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                sections.push_str(&format!(
                    r#"
                        <tr>
                            <td class="warning-msg">{message}</td>
                        </tr>
"#
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
}
