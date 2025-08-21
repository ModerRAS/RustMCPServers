//! 综合测试报告生成器
//! 
//! 这个模块负责生成综合的测试报告，包括：
//! - 单元测试结果
//! - 集成测试结果
//! - E2E测试结果
//! - 性能测试结果
//! - 安全测试结果
//! - 覆盖率报告
//! - 建议和改进措施

use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{TestSuiteResult, TestStatus};

/// 综合测试报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveTestReport {
    pub project_name: String,
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub test_duration: std::time::Duration,
    pub summary: TestSummary,
    pub unit_tests: Option<TestSuiteResult>,
    pub integration_tests: Option<TestSuiteResult>,
    pub e2e_tests: Option<TestSuiteResult>,
    pub performance_results: Option<PerformanceSummary>,
    pub security_results: Option<SecuritySummary>,
    pub coverage_results: Option<CoverageSummary>,
    pub recommendations: Vec<Recommendation>,
    pub metadata: ReportMetadata,
}

/// 测试总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub success_rate: f64,
    pub overall_status: TestStatus,
}

/// 性能测试总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub benchmarks_completed: usize,
    pub average_execution_time_ms: f64,
    pub min_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cache_hit_rate: f64,
    pub performance_grade: PerformanceGrade,
}

/// 性能等级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Average,
    Poor,
}

/// 安全测试总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub vulnerabilities_found: usize,
    pub critical_vulnerabilities: usize,
    pub high_vulnerabilities: usize,
    pub medium_vulnerabilities: usize,
    pub low_vulnerabilities: usize,
    pub security_score: u8,
    pub security_status: SecurityStatus,
}

/// 安全状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityStatus {
    Secure,
    Warning,
    AtRisk,
    Critical,
}

/// 覆盖率总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub line_coverage: f64,
    pub function_coverage: f64,
    pub branch_coverage: f64,
    pub overall_coverage: f64,
    pub coverage_status: CoverageStatus,
}

/// 覆盖率状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoverageStatus {
    Excellent,
    Good,
    NeedsImprovement,
    Poor,
}

/// 建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub action_items: Vec<String>,
}

/// 建议类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    TestCoverage,
    Performance,
    Security,
    CodeQuality,
    Documentation,
    Infrastructure,
}

/// 建议优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    High,
    Medium,
    Low,
}

/// 报告元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub generator_version: String,
    pub test_environment: TestEnvironment,
    pub configuration: ReportConfiguration,
}

/// 测试环境
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironment {
    pub os: String,
    pub rust_version: String,
    pub cpu_info: String,
    pub memory_info: String,
    pub test_tools: Vec<String>,
}

/// 报告配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfiguration {
    pub include_detailed_results: bool,
    pub include_recommendations: bool,
    pub include_charts: bool,
    pub output_formats: Vec<OutputFormat>,
}

/// 输出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Html,
    Markdown,
    Json,
    Pdf,
}

/// 综合测试报告生成器
pub struct ComprehensiveReportGenerator {
    pub project_root: String,
    pub report_dir: String,
    pub config: ReportConfiguration,
}

impl ComprehensiveReportGenerator {
    /// 创建新的综合报告生成器
    pub fn new(project_root: &str, report_dir: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
            report_dir: report_dir.to_string(),
            config: ReportConfiguration {
                include_detailed_results: true,
                include_recommendations: true,
                include_charts: true,
                output_formats: vec![OutputFormat::Html, OutputFormat::Markdown, OutputFormat::Json],
            },
        }
    }

    /// 生成综合测试报告
    pub fn generate_comprehensive_report(
        &self,
        unit_results: Option<TestSuiteResult>,
        integration_results: Option<TestSuiteResult>,
        e2e_results: Option<TestSuiteResult>,
        performance_results: Option<PerformanceSummary>,
        security_results: Option<SecuritySummary>,
        coverage_results: Option<CoverageSummary>,
    ) -> Result<ComprehensiveTestReport, Box<dyn std::error::Error>> {
        
        let summary = self.calculate_test_summary(&unit_results, &integration_results, &e2e_results);
        let recommendations = self.generate_recommendations(
            &summary,
            &performance_results,
            &security_results,
            &coverage_results,
        );

        let report = ComprehensiveTestReport {
            project_name: self.get_project_name(),
            report_id: self.generate_report_id(),
            generated_at: Utc::now(),
            test_duration: std::time::Duration::from_secs(0),
            summary,
            unit_tests: unit_results,
            integration_tests: integration_results,
            e2e_tests: e2e_results,
            performance_results,
            security_results,
            coverage_results,
            recommendations,
            metadata: self.get_report_metadata(),
        };

        Ok(report)
    }

    /// 生成HTML报告
    pub fn generate_html_report(&self, report: &ComprehensiveTestReport) -> Result<String, Box<dyn std::error::Error>> {
        let report_path = format!("{}/comprehensive_report_{}.html", 
                                self.report_dir, 
                                report.generated_at.format("%Y%m%d_%H%M%S"));

        let html_content = self.generate_html_content(report);
        fs::write(&report_path, html_content)?;
        
        Ok(report_path)
    }

    /// 生成Markdown报告
    pub fn generate_markdown_report(&self, report: &ComprehensiveTestReport) -> Result<String, Box<dyn std::error::Error>> {
        let report_path = format!("{}/comprehensive_report_{}.md", 
                                self.report_dir, 
                                report.generated_at.format("%Y%m%d_%H%M%S"));

        let markdown_content = self.generate_markdown_content(report);
        fs::write(&report_path, markdown_content)?;
        
        Ok(report_path)
    }

    /// 生成JSON报告
    pub fn generate_json_report(&self, report: &ComprehensiveTestReport) -> Result<String, Box<dyn std::error::Error>> {
        let report_path = format!("{}/comprehensive_report_{}.json", 
                                self.report_dir, 
                                report.generated_at.format("%Y%m%d_%H%M%S"));

        let json_content = serde_json::to_string_pretty(report)?;
        fs::write(&report_path, json_content)?;
        
        Ok(report_path)
    }

    /// 计算测试总结
    fn calculate_test_summary(
        &self,
        unit_results: &Option<TestSuiteResult>,
        integration_results: &Option<TestSuiteResult>,
        e2e_results: &Option<TestSuiteResult>,
    ) -> TestSummary {
        let mut total_tests = 0;
        let mut passed_tests = 0;
        let mut failed_tests = 0;
        let mut skipped_tests = 0;

        if let Some(unit) = unit_results {
            total_tests += unit.total_tests;
            passed_tests += unit.passed_tests;
            failed_tests += unit.failed_tests;
            skipped_tests += unit.total_tests - unit.passed_tests - unit.failed_tests;
        }

        if let Some(integration) = integration_results {
            total_tests += integration.total_tests;
            passed_tests += integration.passed_tests;
            failed_tests += integration.failed_tests;
            skipped_tests += integration.total_tests - integration.passed_tests - integration.failed_tests;
        }

        if let Some(e2e) = e2e_results {
            total_tests += e2e.total_tests;
            passed_tests += e2e.passed_tests;
            failed_tests += e2e.failed_tests;
            skipped_tests += e2e.total_tests - e2e.passed_tests - e2e.failed_tests;
        }

        let success_rate = if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        let overall_status = if failed_tests == 0 {
            TestStatus::Passed
        } else if success_rate >= 80.0 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        TestSummary {
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            success_rate,
            overall_status,
        }
    }

    /// 生成建议
    fn generate_recommendations(
        &self,
        summary: &TestSummary,
        performance: &Option<PerformanceSummary>,
        security: &Option<SecuritySummary>,
        coverage: &Option<CoverageSummary>,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // 测试覆盖率建议
        if let Some(cov) = coverage {
            if cov.overall_coverage < 80.0 {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::TestCoverage,
                    priority: RecommendationPriority::High,
                    title: "提高测试覆盖率".to_string(),
                    description: format!("当前测试覆盖率为 {:.1}%，建议提高到80%以上", cov.overall_coverage),
                    action_items: vec![
                        "为未覆盖的核心功能添加单元测试".to_string(),
                        "增加边界情况和错误处理的测试".to_string(),
                        "考虑使用覆盖率工具识别未测试的代码".to_string(),
                    ],
                });
            }
        }

        // 性能建议
        if let Some(perf) = performance {
            if matches!(perf.performance_grade, PerformanceGrade::Poor | PerformanceGrade::Average) {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Performance,
                    priority: RecommendationPriority::Medium,
                    title: "优化性能".to_string(),
                    description: "当前性能表现有待提升，建议进行性能优化".to_string(),
                    action_items: vec![
                        "分析性能瓶颈并进行优化".to_string(),
                        "考虑使用缓存提高响应速度".to_string(),
                        "优化数据库查询和算法复杂度".to_string(),
                    ],
                });
            }
        }

        // 安全建议
        if let Some(sec) = security {
            if sec.critical_vulnerabilities > 0 || sec.high_vulnerabilities > 0 {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Security,
                    priority: RecommendationPriority::High,
                    title: "修复安全漏洞".to_string(),
                    description: format!("发现 {} 个关键和高危漏洞，需要立即修复", 
                                       sec.critical_vulnerabilities + sec.high_vulnerabilities),
                    action_items: vec![
                        "修复所有关键和高危漏洞".to_string(),
                        "进行安全代码审查".to_string(),
                        "添加安全测试到CI/CD流程".to_string(),
                    ],
                });
            }
        }

        // 测试失败建议
        if summary.failed_tests > 0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::TestCoverage,
                priority: RecommendationPriority::High,
                title: "修复失败的测试".to_string(),
                description: format!("有 {} 个测试失败，需要修复", summary.failed_tests),
                action_items: vec![
                    "分析失败的测试用例".to_string(),
                    "修复相关代码或更新测试用例".to_string(),
                    "确保所有测试都能稳定运行".to_string(),
                ],
            });
        }

        recommendations
    }

    /// 生成HTML内容
    fn generate_html_content(&self, report: &ComprehensiveTestReport) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>综合测试报告 - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background-color: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .header {{ background-color: #007bff; color: white; padding: 20px; border-radius: 5px; margin-bottom: 20px; }}
        .summary {{ display: flex; justify-content: space-between; margin-bottom: 20px; }}
        .summary-card {{ flex: 1; margin: 0 10px; padding: 15px; border-radius: 5px; text-align: center; }}
        .success {{ background-color: #d4edda; border: 1px solid #c3e6cb; }}
        .warning {{ background-color: #fff3cd; border: 1px solid #ffeaa7; }}
        .danger {{ background-color: #f8d7da; border: 1px solid #f5c6cb; }}
        .info {{ background-color: #d1ecf1; border: 1px solid #bee5eb; }}
        .section {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .recommendation {{ margin: 10px 0; padding: 10px; border-left: 4px solid #007bff; background-color: #f8f9fa; }}
        .high-priority {{ border-left-color: #dc3545; }}
        .medium-priority {{ border-left-color: #ffc107; }}
        .low-priority {{ border-left-color: #28a745; }}
        .progress-bar {{ width: 100%; height: 20px; background-color: #e0e0e0; border-radius: 10px; overflow: hidden; }}
        .progress-fill {{ height: 100%; background-color: #28a745; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>综合测试报告</h1>
            <p>项目: {} | 报告ID: {} | 生成时间: {}</p>
        </div>

        <div class="summary">
            <div class="summary-card info">
                <h3>总测试数</h3>
                <p style="font-size: 24px;">{}</p>
            </div>
            <div class="summary-card success">
                <h3>通过测试</h3>
                <p style="font-size: 24px;">{}</p>
            </div>
            <div class="summary-card danger">
                <h3>失败测试</h3>
                <p style="font-size: 24px;">{}</p>
            </div>
            <div class="summary-card warning">
                <h3>成功率</h3>
                <p style="font-size: 24px;">{:.1}%</p>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: {}%"></div>
                </div>
            </div>
        </div>

        <div class="section">
            <h2>改进建议</h2>
            {}
        </div>
    </div>
</body>
</html>
            "#,
            report.project_name,
            report.project_name,
            report.report_id,
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.summary.total_tests,
            report.summary.passed_tests,
            report.summary.failed_tests,
            report.summary.success_rate,
            report.summary.success_rate,
            self.generate_recommendations_html(&report.recommendations)
        )
    }

    /// 生成Markdown内容
    fn generate_markdown_content(&self, report: &ComprehensiveTestReport) -> String {
        let mut content = String::new();

        content.push_str(&format!(
            "# 综合测试报告\n\n\
            **项目:** {}\n\
            **报告ID:** {}\n\
            **生成时间:** {}\n\n",
            report.project_name,
            report.report_id,
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        content.push_str("## 测试总结\n\n");
        content.push_str(&format!(
            "| 指标 | 数值 |\n|------|------|\n\
            | 总测试数 | {} |\n\
            | 通过测试 | {} |\n\
            | 失败测试 | {} |\n\
            | 跳过测试 | {} |\n\
            | 成功率 | {:.1}% |\n\
            | 整体状态 | {} |\n\n",
            report.summary.total_tests,
            report.summary.passed_tests,
            report.summary.failed_tests,
            report.summary.skipped_tests,
            report.summary.success_rate,
            self.format_test_status(&report.summary.overall_status)
        ));

        content.push_str("## 改进建议\n\n");
        for recommendation in &report.recommendations {
            content.push_str(&format!(
                "### {}\n\n\
                **类别:** {} | **优先级:** {}\n\n\
                {}\n\n\
                **行动项:**\n{}\n\n",
                recommendation.title,
                self.format_recommendation_category(&recommendation.category),
                self.format_recommendation_priority(&recommendation.priority),
                recommendation.description,
                recommendation.action_items.iter()
                    .map(|item| format!("- {}", item))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        content
    }

    /// 获取项目名称
    fn get_project_name(&self) -> String {
        Path::new(&self.project_root)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Project")
            .to_string()
    }

    /// 生成报告ID
    fn generate_report_id(&self) -> String {
        format!("RPT-{}", Utc::now().format("%Y%m%d%H%M%S"))
    }

    /// 获取报告元数据
    fn get_report_metadata(&self) -> ReportMetadata {
        ReportMetadata {
            generator_version: "1.0.0".to_string(),
            test_environment: TestEnvironment {
                os: std::env::consts::OS.to_string(),
                rust_version: env!("CARGO_PKG_VERSION").to_string(),
                cpu_info: "Unknown".to_string(),
                memory_info: "Unknown".to_string(),
                test_tools: vec![
                    "cargo".to_string(),
                    "criterion".to_string(),
                    "tarpaulin".to_string(),
                ],
            },
            configuration: self.config.clone(),
        }
    }

    /// 生成建议HTML
    fn generate_recommendations_html(&self, recommendations: &[Recommendation]) -> String {
        let mut html = String::new();
        
        for recommendation in recommendations {
            html.push_str(&format!(
                "<div class=\"recommendation {}-priority\">\n\
                    <h3>{}</h3>\n\
                    <p><strong>类别:</strong> {} | <strong>优先级:</strong> {}</p>\n\
                    <p>{}</p>\n\
                    <h4>行动项:</h4>\n\
                    <ul>\n{}\n\
                    </ul>\n\
                </div>",
                self.format_recommendation_priority_class(&recommendation.priority),
                recommendation.title,
                self.format_recommendation_category(&recommendation.category),
                self.format_recommendation_priority(&recommendation.priority),
                recommendation.description,
                recommendation.action_items.iter()
                    .map(|item| format!("<li>{}</li>", item))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        
        html
    }

    /// 格式化测试状态
    fn format_test_status(&self, status: &TestStatus) -> &'static str {
        match status {
            TestStatus::Passed => "✅ 通过",
            TestStatus::Failed => "❌ 失败",
            TestStatus::Skipped => "⏭️ 跳过",
        }
    }

    /// 格式化建议类别
    fn format_recommendation_category(&self, category: &RecommendationCategory) -> &'static str {
        match category {
            RecommendationCategory::TestCoverage => "测试覆盖率",
            RecommendationCategory::Performance => "性能",
            RecommendationCategory::Security => "安全",
            RecommendationCategory::CodeQuality => "代码质量",
            RecommendationCategory::Documentation => "文档",
            RecommendationCategory::Infrastructure => "基础设施",
        }
    }

    /// 格式化建议优先级
    fn format_recommendation_priority(&self, priority: &RecommendationPriority) -> &'static str {
        match priority {
            RecommendationPriority::High => "高",
            RecommendationPriority::Medium => "中",
            RecommendationPriority::Low => "低",
        }
    }

    /// 格式化建议优先级样式
    fn format_recommendation_priority_class(&self, priority: &RecommendationPriority) -> &'static str {
        match priority {
            RecommendationPriority::High => "high",
            RecommendationPriority::Medium => "medium",
            RecommendationPriority::Low => "low",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_comprehensive_report_generator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = ComprehensiveReportGenerator::new("/test/project", temp_dir.path().to_str().unwrap());
        
        assert_eq!(generator.project_root, "/test/project");
        assert_eq!(generator.report_dir, temp_dir.path().to_str().unwrap());
    }

    #[test]
    fn test_summary_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = ComprehensiveReportGenerator::new("/test/project", temp_dir.path().to_str().unwrap());
        
        let unit_results = TestSuiteResult {
            suite_name: "unit".to_string(),
            total_tests: 10,
            passed_tests: 8,
            failed_tests: 2,
            execution_time_ms: 1000,
            test_results: Vec::new(),
        };

        let integration_results = TestSuiteResult {
            suite_name: "integration".to_string(),
            total_tests: 5,
            passed_tests: 4,
            failed_tests: 1,
            execution_time_ms: 2000,
            test_results: Vec::new(),
        };

        let summary = generator.calculate_test_summary(&Some(unit_results), &Some(integration_results), &None);
        
        assert_eq!(summary.total_tests, 15);
        assert_eq!(summary.passed_tests, 12);
        assert_eq!(summary.failed_tests, 3);
        assert_eq!(summary.success_rate, 80.0);
    }

    #[test]
    fn test_recommendation_generation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = ComprehensiveReportGenerator::new("/test/project", temp_dir.path().to_str().unwrap());
        
        let summary = TestSummary {
            total_tests: 10,
            passed_tests: 8,
            failed_tests: 2,
            skipped_tests: 0,
            success_rate: 80.0,
            overall_status: TestStatus::Passed,
        };

        let coverage = CoverageSummary {
            line_coverage: 65.0,
            function_coverage: 70.0,
            branch_coverage: 60.0,
            overall_coverage: 65.0,
            coverage_status: CoverageStatus::NeedsImprovement,
        };

        let recommendations = generator.generate_recommendations(&summary, &None, &None, &Some(coverage));
        
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.title.contains("提高测试覆盖率")));
    }

    #[test]
    fn test_markdown_report_generation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = ComprehensiveReportGenerator::new("/test/project", temp_dir.path().to_str().unwrap());
        
        let summary = TestSummary {
            total_tests: 10,
            passed_tests: 10,
            failed_tests: 0,
            skipped_tests: 0,
            success_rate: 100.0,
            overall_status: TestStatus::Passed,
        };

        let report = ComprehensiveTestReport {
            project_name: "Test Project".to_string(),
            report_id: "TEST-123".to_string(),
            generated_at: Utc::now(),
            test_duration: std::time::Duration::from_secs(60),
            summary,
            unit_tests: None,
            integration_tests: None,
            e2e_tests: None,
            performance_results: None,
            security_results: None,
            coverage_results: None,
            recommendations: Vec::new(),
            metadata: generator.get_report_metadata(),
        };

        let markdown_content = generator.generate_markdown_content(&report);
        
        assert!(markdown_content.contains("综合测试报告"));
        assert!(markdown_content.contains("Test Project"));
        assert!(markdown_content.contains("测试总结"));
    }
}