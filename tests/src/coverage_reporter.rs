//! 测试覆盖率报告生成器
//! 
//! 这个模块负责生成详细的测试覆盖率报告，包括：
//! - 代码覆盖率分析
//! - 测试覆盖率统计
//! - 覆盖率趋势分析
//! - 覆盖率改进建议

use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 覆盖率数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageData {
    pub total_lines: u32,
    pub covered_lines: u32,
    pub total_functions: u32,
    pub covered_functions: u32,
    pub total_branches: u32,
    pub covered_branches: u32,
    pub file_coverage: HashMap<String, FileCoverage>,
    pub timestamp: DateTime<Utc>,
}

/// 文件覆盖率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub file_path: String,
    pub total_lines: u32,
    pub covered_lines: u32,
    pub total_functions: u32,
    pub covered_functions: u32,
    pub total_branches: u32,
    pub covered_branches: u32,
    pub coverage_percentage: f64,
    pub uncovered_lines: Vec<u32>,
    pub partially_covered_lines: Vec<LineCoverage>,
}

/// 行覆盖率详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineCoverage {
    pub line_number: u32,
    pub hit_count: u32,
    pub is_branch: bool,
}

/// 覆盖率报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub project_name: String,
    pub overall_coverage: CoverageMetrics,
    pub file_coverage: HashMap<String, FileCoverage>,
    pub trends: CoverageTrends,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// 覆盖率指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub line_coverage: f64,
    pub function_coverage: f64,
    pub branch_coverage: f64,
    pub overall_coverage: f64,
    pub total_lines: u32,
    pub covered_lines: u32,
    pub total_functions: u32,
    pub covered_functions: u32,
    pub total_branches: u32,
    pub covered_branches: u32,
}

/// 覆盖率趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTrends {
    pub line_coverage_trend: Vec<f64>,
    pub function_coverage_trend: Vec<f64>,
    pub branch_coverage_trend: Vec<f64>,
    pub timestamps: Vec<DateTime<Utc>>,
}

/// 覆盖率报告生成器
pub struct CoverageReportGenerator {
    pub project_root: String,
    pub coverage_data: Vec<CoverageData>,
    pub report_dir: String,
}

impl CoverageReportGenerator {
    /// 创建新的覆盖率报告生成器
    pub fn new(project_root: &str, report_dir: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
            coverage_data: Vec::new(),
            report_dir: report_dir.to_string(),
        }
    }

    /// 解析覆盖率数据
    pub fn parse_coverage_data(&mut self, _coverage_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在实际实现中，这里会解析真实的覆盖率数据文件
        // 例如：tarpaulin、grcov、lcov等工具生成的文件
        
        let coverage_data = self.generate_mock_coverage_data();
        self.coverage_data.push(coverage_data);
        
        Ok(())
    }

    /// 生成覆盖率报告
    pub fn generate_coverage_report(&self) -> Result<CoverageReport, Box<dyn std::error::Error>> {
        let overall_coverage = self.calculate_overall_coverage();
        let trends = self.calculate_coverage_trends();
        let recommendations = self.generate_coverage_recommendations(&overall_coverage);

        let report = CoverageReport {
            project_name: self.project_name(),
            overall_coverage,
            file_coverage: self.get_latest_file_coverage(),
            trends,
            recommendations,
            timestamp: Utc::now(),
        };

        Ok(report)
    }

    /// 生成HTML覆盖率报告
    pub fn generate_html_report(&self, report: &CoverageReport) -> Result<String, Box<dyn std::error::Error>> {
        let report_path = format!("{}/coverage_report_{}.html", 
                                self.report_dir, 
                                report.timestamp.format("%Y%m%d_%H%M%S"));

        let html_content = self.generate_html_content(report);

        fs::write(&report_path, html_content)?;
        
        Ok(report_path)
    }

    /// 生成Markdown覆盖率报告
    pub fn generate_markdown_report(&self, report: &CoverageReport) -> Result<String, Box<dyn std::error::Error>> {
        let report_path = format!("{}/coverage_report_{}.md", 
                                self.report_dir, 
                                report.timestamp.format("%Y%m%d_%H%M%S"));

        let markdown_content = self.generate_markdown_content(report);

        fs::write(&report_path, markdown_content)?;
        
        Ok(report_path)
    }

    /// 生成JSON覆盖率报告
    pub fn generate_json_report(&self, report: &CoverageReport) -> Result<String, Box<dyn std::error::Error>> {
        let report_path = format!("{}/coverage_report_{}.json", 
                                self.report_dir, 
                                report.timestamp.format("%Y%m%d_%H%M%S"));

        let json_content = serde_json::to_string_pretty(report)?;

        fs::write(&report_path, json_content)?;
        
        Ok(report_path)
    }

    /// 计算总体覆盖率
    fn calculate_overall_coverage(&self) -> CoverageMetrics {
        if let Some(latest_data) = self.coverage_data.last() {
            let line_coverage = (latest_data.covered_lines as f64 / latest_data.total_lines as f64) * 100.0;
            let function_coverage = (latest_data.covered_functions as f64 / latest_data.total_functions as f64) * 100.0;
            let branch_coverage = (latest_data.covered_branches as f64 / latest_data.total_branches as f64) * 100.0;
            let overall_coverage = (line_coverage + function_coverage + branch_coverage) / 3.0;

            CoverageMetrics {
                line_coverage,
                function_coverage,
                branch_coverage,
                overall_coverage,
                total_lines: latest_data.total_lines,
                covered_lines: latest_data.covered_lines,
                total_functions: latest_data.total_functions,
                covered_functions: latest_data.covered_functions,
                total_branches: latest_data.total_branches,
                covered_branches: latest_data.covered_branches,
            }
        } else {
            CoverageMetrics {
                line_coverage: 0.0,
                function_coverage: 0.0,
                branch_coverage: 0.0,
                overall_coverage: 0.0,
                total_lines: 0,
                covered_lines: 0,
                total_functions: 0,
                covered_functions: 0,
                total_branches: 0,
                covered_branches: 0,
            }
        }
    }

    /// 计算覆盖率趋势
    fn calculate_coverage_trends(&self) -> CoverageTrends {
        let mut line_trend = Vec::new();
        let mut function_trend = Vec::new();
        let mut branch_trend = Vec::new();
        let mut timestamps = Vec::new();

        for data in &self.coverage_data {
            let line_coverage = (data.covered_lines as f64 / data.total_lines as f64) * 100.0;
            let function_coverage = (data.covered_functions as f64 / data.total_functions as f64) * 100.0;
            let branch_coverage = (data.covered_branches as f64 / data.total_branches as f64) * 100.0;

            line_trend.push(line_coverage);
            function_trend.push(function_coverage);
            branch_trend.push(branch_coverage);
            timestamps.push(data.timestamp);
        }

        CoverageTrends {
            line_coverage_trend: line_trend,
            function_coverage_trend: function_trend,
            branch_coverage_trend: branch_trend,
            timestamps,
        }
    }

    /// 生成覆盖率改进建议
    fn generate_coverage_recommendations(&self, metrics: &CoverageMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于行覆盖率的建议
        if metrics.line_coverage < 80.0 {
            recommendations.push(format!(
                "行覆盖率较低 ({:.1}%)，建议添加更多单元测试覆盖核心业务逻辑",
                metrics.line_coverage
            ));
        }

        // 基于函数覆盖率的建议
        if metrics.function_coverage < 85.0 {
            recommendations.push(format!(
                "函数覆盖率较低 ({:.1}%)，建议测试所有公共函数和方法",
                metrics.function_coverage
            ));
        }

        // 基于分支覆盖率的建议
        if metrics.branch_coverage < 75.0 {
            recommendations.push(format!(
                "分支覆盖率较低 ({:.1}%)，建议添加条件分支和错误处理的测试",
                metrics.branch_coverage
            ));
        }

        // 基于总体覆盖率的建议
        if metrics.overall_coverage < 80.0 {
            recommendations.push(format!(
                "总体覆盖率低于目标 ({:.1}%)，建议制定测试覆盖率提升计划",
                metrics.overall_coverage
            ));
        }

        // 通用建议
        if metrics.overall_coverage >= 80.0 {
            recommendations.push("覆盖率已达到良好水平，建议重点关注边界情况和错误处理测试".to_string());
        }

        if metrics.overall_coverage >= 90.0 {
            recommendations.push("覆盖率优秀！建议保持现有测试质量并关注测试的可维护性".to_string());
        }

        recommendations
    }

    /// 获取项目名称
    fn project_name(&self) -> String {
        Path::new(&self.project_root)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Project")
            .to_string()
    }

    /// 获取最新的文件覆盖率
    fn get_latest_file_coverage(&self) -> HashMap<String, FileCoverage> {
        self.coverage_data
            .last()
            .map(|data| data.file_coverage.clone())
            .unwrap_or_default()
    }

    /// 生成HTML内容
    fn generate_html_content(&self, report: &CoverageReport) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>测试覆盖率报告 - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f8f9fa; padding: 20px; border-radius: 5px; }}
        .metric {{ display: inline-block; margin: 10px; padding: 15px; border-radius: 5px; }}
        .metric.excellent {{ background-color: #d4edda; }}
        .metric.good {{ background-color: #d1ecf1; }}
        .metric.warning {{ background-color: #fff3cd; }}
        .metric.poor {{ background-color: #f8d7da; }}
        .file-coverage {{ margin: 10px 0; padding: 10px; border: 1px solid #ddd; border-radius: 3px; }}
        .recommendations {{ background-color: #e9ecef; padding: 15px; border-radius: 5px; }}
        .progress-bar {{ width: 200px; height: 20px; background-color: #e0e0e0; border-radius: 10px; overflow: hidden; }}
        .progress-fill {{ height: 100%; background-color: #4CAF50; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>测试覆盖率报告</h1>
        <p>项目: {}</p>
        <p>生成时间: {}</p>
    </div>

    <div class="metrics">
        <div class="metric {}">
            <h3>总体覆盖率</h3>
            <div class="progress-bar">
                <div class="progress-fill" style="width: {}%"></div>
            </div>
            <p>{:.1}%</p>
        </div>
        <div class="metric {}">
            <h3>行覆盖率</h3>
            <div class="progress-bar">
                <div class="progress-fill" style="width: {}%"></div>
            </div>
            <p>{:.1}%</p>
        </div>
        <div class="metric {}">
            <h3>函数覆盖率</h3>
            <div class="progress-bar">
                <div class="progress-fill" style="width: {}%"></div>
            </div>
            <p>{:.1}%</p>
        </div>
        <div class="metric {}">
            <h3>分支覆盖率</h3>
            <div class="progress-bar">
                <div class="progress-fill" style="width: {}%"></div>
            </div>
            <p>{:.1}%</p>
        </div>
    </div>

    <div class="file-coverage">
        <h2>文件覆盖率详情</h2>
        {}
    </div>

    <div class="recommendations">
        <h2>改进建议</h2>
        <ul>
            {}
        </ul>
    </div>
</body>
</html>
            "#,
            report.project_name,
            report.project_name,
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.get_coverage_class(report.overall_coverage.overall_coverage),
            report.overall_coverage.overall_coverage,
            report.overall_coverage.overall_coverage,
            self.get_coverage_class(report.overall_coverage.line_coverage),
            report.overall_coverage.line_coverage,
            report.overall_coverage.line_coverage,
            self.get_coverage_class(report.overall_coverage.function_coverage),
            report.overall_coverage.function_coverage,
            report.overall_coverage.function_coverage,
            self.get_coverage_class(report.overall_coverage.branch_coverage),
            report.overall_coverage.branch_coverage,
            report.overall_coverage.branch_coverage,
            self.generate_file_coverage_html(&report.file_coverage),
            self.generate_recommendations_html(&report.recommendations)
        )
    }

    /// 生成Markdown内容
    fn generate_markdown_content(&self, report: &CoverageReport) -> String {
        let mut content = String::new();

        content.push_str(&format!(
            "# 测试覆盖率报告\n\n\
            **项目:** {}\n\
            **生成时间:** {}\n\n",
            report.project_name,
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        content.push_str("## 覆盖率概览\n\n");
        content.push_str(&format!(
            "| 指标 | 覆盖率 | 状态 |\n|------|--------|------|\n\
            | 总体覆盖率 | {:.1}% | {} |\n\
            | 行覆盖率 | {:.1}% | {} |\n\
            | 函数覆盖率 | {:.1}% | {} |\n\
            | 分支覆盖率 | {:.1}% | {} |\n\n",
            report.overall_coverage.overall_coverage,
            self.get_coverage_status(report.overall_coverage.overall_coverage),
            report.overall_coverage.line_coverage,
            self.get_coverage_status(report.overall_coverage.line_coverage),
            report.overall_coverage.function_coverage,
            self.get_coverage_status(report.overall_coverage.function_coverage),
            report.overall_coverage.branch_coverage,
            self.get_coverage_status(report.overall_coverage.branch_coverage)
        ));

        content.push_str("## 文件覆盖率详情\n\n");
        for (file_path, file_coverage) in &report.file_coverage {
            content.push_str(&format!(
                "### {}\n\n\
                - **行覆盖率:** {:.1}% ({}/{})\n\
                - **函数覆盖率:** {:.1}% ({}/{})\n\
                - **分支覆盖率:** {:.1}% ({}/{})\n\n",
                file_path,
                file_coverage.coverage_percentage,
                file_coverage.covered_lines,
                file_coverage.total_lines,
                (file_coverage.covered_functions as f64 / file_coverage.total_functions as f64) * 100.0,
                file_coverage.covered_functions,
                file_coverage.total_functions,
                (file_coverage.covered_branches as f64 / file_coverage.total_branches as f64) * 100.0,
                file_coverage.covered_branches,
                file_coverage.total_branches
            ));
        }

        content.push_str("## 改进建议\n\n");
        for recommendation in &report.recommendations {
            content.push_str(&format!("- {}\n", recommendation));
        }
        content.push('\n');

        content
    }

    /// 获取覆盖率等级样式
    fn get_coverage_class(&self, coverage: f64) -> &'static str {
        if coverage >= 90.0 {
            "excellent"
        } else if coverage >= 80.0 {
            "good"
        } else if coverage >= 70.0 {
            "warning"
        } else {
            "poor"
        }
    }

    /// 获取覆盖率状态
    fn get_coverage_status(&self, coverage: f64) -> &'static str {
        if coverage >= 90.0 {
            "✅ 优秀"
        } else if coverage >= 80.0 {
            "✅ 良好"
        } else if coverage >= 70.0 {
            "⚠️ 一般"
        } else {
            "❌ 需要改进"
        }
    }

    /// 生成文件覆盖率HTML
    fn generate_file_coverage_html(&self, file_coverage: &HashMap<String, FileCoverage>) -> String {
        let mut html = String::new();
        
        for (file_path, coverage) in file_coverage {
            html.push_str(&format!(
                "<div class=\"file-coverage\">\n\
                    <h3>{}</h3>\n\
                    <p>行覆盖率: {:.1}% | 函数覆盖率: {:.1}% | 分支覆盖率: {:.1}%</p>\n\
                </div>",
                file_path,
                coverage.coverage_percentage,
                (coverage.covered_functions as f64 / coverage.total_functions as f64) * 100.0,
                (coverage.covered_branches as f64 / coverage.total_branches as f64) * 100.0
            ));
        }
        
        html
    }

    /// 生成建议HTML
    fn generate_recommendations_html(&self, recommendations: &[String]) -> String {
        let mut html = String::new();
        
        for recommendation in recommendations {
            html.push_str(&format!("<li>{}</li>", recommendation));
        }
        
        html
    }

    /// 生成模拟覆盖率数据（用于测试）
    fn generate_mock_coverage_data(&self) -> CoverageData {
        let mut file_coverage = HashMap::new();
        
        // 添加一些模拟的文件覆盖率数据
        file_coverage.insert("src/lib.rs".to_string(), FileCoverage {
            file_path: "src/lib.rs".to_string(),
            total_lines: 150,
            covered_lines: 135,
            total_functions: 20,
            covered_functions: 18,
            total_branches: 30,
            covered_branches: 25,
            coverage_percentage: 90.0,
            uncovered_lines: vec![10, 25, 40],
            partially_covered_lines: vec![
                LineCoverage { line_number: 15, hit_count: 3, is_branch: true },
                LineCoverage { line_number: 30, hit_count: 1, is_branch: false },
            ],
        });

        file_coverage.insert("src/main.rs".to_string(), FileCoverage {
            file_path: "src/main.rs".to_string(),
            total_lines: 80,
            covered_lines: 60,
            total_functions: 8,
            covered_functions: 6,
            total_branches: 12,
            covered_branches: 8,
            coverage_percentage: 75.0,
            uncovered_lines: vec![5, 20, 35, 50],
            partially_covered_lines: vec![
                LineCoverage { line_number: 10, hit_count: 2, is_branch: true },
            ],
        });

        CoverageData {
            total_lines: 230,
            covered_lines: 195,
            total_functions: 28,
            covered_functions: 24,
            total_branches: 42,
            covered_branches: 33,
            file_coverage,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_coverage_report_generator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = CoverageReportGenerator::new("/test/project", temp_dir.path().to_str().unwrap());
        
        assert_eq!(generator.project_root, "/test/project");
        assert_eq!(generator.report_dir, temp_dir.path().to_str().unwrap());
    }

    #[test]
    fn test_coverage_metrics_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let mut generator = CoverageReportGenerator::new("/test/project", temp_dir.path().to_str().unwrap());
        
        // 添加模拟数据
        generator.coverage_data.push(generator.generate_mock_coverage_data());
        
        let metrics = generator.calculate_overall_coverage();
        
        assert!(metrics.line_coverage > 0.0);
        assert!(metrics.function_coverage > 0.0);
        assert!(metrics.branch_coverage > 0.0);
        assert!(metrics.overall_coverage > 0.0);
    }

    #[test]
    fn test_coverage_recommendations() {
        let temp_dir = TempDir::new().unwrap();
        let generator = CoverageReportGenerator::new("/test/project", temp_dir.path().to_str().unwrap());
        
        let metrics = CoverageMetrics {
            line_coverage: 65.0,
            function_coverage: 70.0,
            branch_coverage: 60.0,
            overall_coverage: 65.0,
            total_lines: 100,
            covered_lines: 65,
            total_functions: 20,
            covered_functions: 14,
            total_branches: 30,
            covered_branches: 18,
        };
        
        let recommendations = generator.generate_coverage_recommendations(&metrics);
        
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("行覆盖率较低")));
        assert!(recommendations.iter().any(|r| r.contains("函数覆盖率较低")));
    }

    #[test]
    fn test_markdown_report_generation() {
        let temp_dir = TempDir::new().unwrap();
        let mut generator = CoverageReportGenerator::new("/test/project", temp_dir.path().to_str().unwrap());
        
        generator.coverage_data.push(generator.generate_mock_coverage_data());
        
        let report = generator.generate_coverage_report().unwrap();
        let markdown_content = generator.generate_markdown_content(&report);
        
        assert!(markdown_content.contains("测试覆盖率报告"));
        assert!(markdown_content.contains("总体覆盖率"));
        assert!(markdown_content.contains("行覆盖率"));
        assert!(markdown_content.contains("改进建议"));
    }
}