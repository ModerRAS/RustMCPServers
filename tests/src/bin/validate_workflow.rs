use std::env;
use std::process;
use github_actions_tests::*;

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <workflow-file>", args[0]);
        process::exit(1);
    }
    
    let workflow_file = &args[1];
    
    // 验证工作流文件
    match validate_github_workflow(workflow_file).await {
        Ok(_) => {
            println!("✅ Workflow validation completed successfully");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Workflow validation failed: {}", e);
            process::exit(1);
        }
    }
}

async fn validate_github_workflow(workflow_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating GitHub Actions workflow: {}", workflow_file);
    
    // 1. 工作流验证
    println!("📋 Running workflow validation...");
    let validator = WorkflowValidator::new(workflow_file)?;
    let validation_result = validator.validate();
    
    // 2. 安全测试
    println!("🛡️ Running security tests...");
    let security_tester = SecurityTester::new(workflow_file)?;
    let security_result = security_tester.run_security_tests();
    
    // 3. 性能测试
    println!("⚡ Running performance tests...");
    let performance_tester = PerformanceTester::new(3, 1);
    let performance_results = performance_tester.test_workflow_performance(workflow_file).await.map_err(|e| format!("Performance test failed: {}", e))?;
    
    // 4. 生成报告
    println!("📊 Generating validation report...");
    let report = generate_validation_report(&validation_result, &security_result, &performance_results);
    
    // 5. 输出结果
    println!("\n🎯 Validation Results:");
    println!("======================");
    println!("📋 Workflow Validation: {}", if validation_result.is_valid { "✅ Valid" } else { "❌ Invalid" });
    println!("🛡️ Security Score: {}/100 ({})", security_result.score, if security_result.is_secure { "Secure" } else { "Vulnerable" });
    println!("⚡ Performance Tests: {} completed", performance_results.len());
    
    if !validation_result.is_valid {
        println!("\n❌ Workflow Validation Issues:");
        for error in &validation_result.errors {
            println!("  - ERROR: {}", error);
        }
        for warning in &validation_result.warnings {
            println!("  - WARNING: {}", warning);
        }
    }
    
    if !security_result.is_secure {
        println!("\n🛡️ Security Vulnerabilities Found:");
        for vuln in &security_result.vulnerabilities {
            println!("  - {:?}: {} - {}", vuln.severity, vuln.category, vuln.title);
        }
    }
    
    // 6. 保存报告
    let report_path = format!("{}_validation_report.md", workflow_file.replace(".yml", "").replace("/", "_"));
    std::fs::write(&report_path, report)?;
    println!("📄 Validation report saved to: {}", report_path);
    
    // 7. 判断整体状态
    if validation_result.is_valid && security_result.is_secure {
        Ok(())
    } else {
        Err("Workflow validation failed".into())
    }
}

fn generate_validation_report(
    validation_result: &WorkflowValidationResult,
    security_result: &SecurityTestResult,
    performance_results: &Vec<PerformanceTestResult>,
) -> String {
    let mut report = String::new();
    report.push_str(&format!("# GitHub Actions Workflow Validation Report\n\n"));
    report.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    // 工作流验证结果
    report.push_str("## Workflow Validation\n\n");
    report.push_str(&format!("- **Status:** {}\n", if validation_result.is_valid { "✅ Valid" } else { "❌ Invalid" }));
    report.push_str(&format!("- **Errors:** {}\n", validation_result.errors.len()));
    report.push_str(&format!("- **Warnings:** {}\n", validation_result.warnings.len()));
    report.push_str(&format!("- **Info:** {}\n", validation_result.info.len()));
    
    if !validation_result.errors.is_empty() {
        report.push_str("\n### Errors\n\n");
        for error in &validation_result.errors {
            report.push_str(&format!("- {}\n", error));
        }
        report.push_str("\n");
    }
    
    if !validation_result.warnings.is_empty() {
        report.push_str("### Warnings\n\n");
        for warning in &validation_result.warnings {
            report.push_str(&format!("- {}\n", warning));
        }
        report.push_str("\n");
    }
    
    // 安全测试结果
    report.push_str("## Security Analysis\n\n");
    report.push_str(&format!("- **Security Score:** {}/100\n", security_result.score));
    report.push_str(&format!("- **Status:** {}\n", if security_result.is_secure { "✅ Secure" } else { "❌ Vulnerable" }));
    report.push_str(&format!("- **Vulnerabilities Found:** {}\n", security_result.vulnerabilities.len()));
    
    if !security_result.vulnerabilities.is_empty() {
        report.push_str("\n### Vulnerabilities\n\n");
        for vuln in &security_result.vulnerabilities {
            report.push_str(&format!("#### {:?} - {}\n\n", vuln.severity, vuln.title));
            report.push_str(&format!("- **Category:** {}\n", vuln.category));
            report.push_str(&format!("- **Description:** {}\n", vuln.description));
            report.push_str(&format!("- **Location:** {}\n", vuln.location));
            report.push_str(&format!("- **Recommendation:** {}\n\n", vuln.recommendation));
        }
    }
    
    // 性能测试结果
    report.push_str("## Performance Analysis\n\n");
    report.push_str(&format!("- **Test Runs:** {}\n", performance_results.len()));
    
    if !performance_results.is_empty() {
        let successful_runs: Vec<_> = performance_results.iter().filter(|r| r.success).collect();
        let failed_runs: Vec<_> = performance_results.iter().filter(|r| !r.success).collect();
        
        report.push_str(&format!("- **Successful Runs:** {}\n", successful_runs.len()));
        report.push_str(&format!("- **Failed Runs:** {}\n", failed_runs.len()));
        
        if !successful_runs.is_empty() {
            let avg_time: f64 = successful_runs.iter().map(|r| r.execution_time_ms).sum::<u64>() as f64 / successful_runs.len() as f64;
            report.push_str(&format!("- **Average Execution Time:** {:.2}ms\n", avg_time));
        }
        
        if !failed_runs.is_empty() {
            report.push_str("\n### Failed Runs\n\n");
            for result in failed_runs {
                report.push_str(&format!("- **Test:** {}\n", result.test_name));
                report.push_str(&format!("- **Error:** Check logs for details\n\n"));
            }
        }
    }
    
    // 建议
    report.push_str("## Recommendations\n\n");
    
    if !validation_result.is_valid {
        report.push_str("### Workflow Fixes\n\n");
        for error in &validation_result.errors {
            report.push_str(&format!("- Fix error: {}\n", error));
        }
        for warning in &validation_result.warnings {
            report.push_str(&format!("- Address warning: {}\n", warning));
        }
        report.push_str("\n");
    }
    
    if !security_result.is_secure {
        report.push_str("### Security Improvements\n\n");
        for rec in &security_result.recommendations {
            report.push_str(&format!("- {}\n", rec));
        }
        report.push_str("\n");
    }
    
    // 总结
    report.push_str("## Summary\n\n");
    let overall_status = if validation_result.is_valid && security_result.is_secure {
        "✅ All checks passed"
    } else {
        "❌ Issues found that need attention"
    };
    report.push_str(&format!("**Overall Status:** {}\n\n", overall_status));
    
    report
}