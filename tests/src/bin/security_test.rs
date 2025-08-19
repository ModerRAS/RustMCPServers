use std::env;
use std::process;
use github_actions_tests::*;

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <workflow-file> [--output-format <json|markdown>]", args[0]);
        process::exit(1);
    }
    
    let workflow_file = &args[1];
    let output_format = args.get(3).map(|s| s.as_str()).unwrap_or("markdown");
    
    // 运行安全测试
    match run_security_test(workflow_file, output_format).await {
        Ok(_) => {
            println!("✅ Security test completed successfully");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Security test failed: {}", e);
            process::exit(1);
        }
    }
}

async fn run_security_test(workflow_file: &str, output_format: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Running security test for GitHub Actions workflow: {}", workflow_file);
    
    // 1. 运行安全测试
    println!("🛡️  Analyzing security vulnerabilities...");
    let security_tester = SecurityTester::new(workflow_file)?;
    let security_result = security_tester.run_security_tests();
    
    // 2. 生成详细的安全报告
    println!("📋 Generating security report...");
    let report = security_tester.generate_security_report(&security_result);
    
    // 3. 输出结果
    println!("\n🎯 Security Test Results:");
    println!("================================");
    println!("📊 Security Score: {}/100", security_result.score);
    println!("🔒 Overall Status: {}", 
        if security_result.is_secure { "✅ SECURE" } else { "❌ VULNERABLE" });
    
    if !security_result.vulnerabilities.is_empty() {
        println!("\n⚠️  Vulnerabilities Found:");
        println!("------------------------");
        
        let by_severity = vec![
            (Severity::Critical, "Critical", "🔴"),
            (Severity::High, "High", "🟠"),
            (Severity::Medium, "Medium", "🟡"),
            (Severity::Low, "Low", "🔵"),
            (Severity::Info, "Info", "⚪"),
        ];
        
        for (severity, label, icon) in by_severity {
            let vulns: Vec<_> = security_result.vulnerabilities.iter()
                .filter(|v| v.severity == severity)
                .collect();
            
            if !vulns.is_empty() {
                println!("{} {} ({}):", icon, label, vulns.len());
                for vuln in vulns {
                    println!("  - {}: {}", vuln.category, vuln.description);
                    println!("    Location: {}", vuln.location);
                    println!("    Fix: {}", vuln.recommendation);
                    println!();
                }
            }
        }
    } else {
        println!("\n✅ No vulnerabilities found!");
    }
    
    // 4. 根据输出格式保存报告
    match output_format {
        "json" => {
            let json_report = serde_json::to_string_pretty(&security_result)?;
            let report_path = format!("{}_security_report.json", 
                workflow_file.replace(".yml", "").replace("/", "_"));
            std::fs::write(&report_path, json_report)?;
            println!("📄 JSON report saved to: {}", report_path);
        }
        "markdown" => {
            let report_path = format!("{}_security_report.md", 
                workflow_file.replace(".yml", "").replace("/", "_"));
            std::fs::write(&report_path, report)?;
            println!("📄 Markdown report saved to: {}", report_path);
        }
        _ => {
            return Err("Unsupported output format. Use 'json' or 'markdown'".into());
        }
    }
    
    // 5. 生成安全建议
    println!("\n💡 Security Recommendations:");
    println!("===========================");
    
    if security_result.score >= 90 {
        println!("🎉 Excellent security posture!");
        println!("   - Continue following security best practices");
        println!("   - Consider regular security audits");
    } else if security_result.score >= 70 {
        println!("✅ Good security posture with room for improvement:");
        for rec in &security_result.recommendations {
            println!("   - {}", rec);
        }
    } else {
        println!("🚨 Security posture needs immediate attention:");
        println!("   - Address critical and high severity vulnerabilities first");
        println!("   - Implement security best practices");
        println!("   - Consider security training for team members");
        
        for rec in &security_result.recommendations {
            println!("   - {}", rec);
        }
    }
    
    // 6. 检查是否达到安全标准
    println!("\n🔍 Security Standards Compliance:");
    println!("==================================");
    
    let standards = vec![
        ("No hardcoded secrets", !has_hardcoded_secrets(&security_result)),
        ("Minimal permissions", has_minimal_permissions(&security_result)),
        ("Updated action versions", has_updated_actions(&security_result)),
        ("No critical vulnerabilities", !has_critical_vulnerabilities(&security_result)),
        ("No high vulnerabilities", !has_high_vulnerabilities(&security_result)),
    ];
    
    for (standard, compliant) in standards {
        println!("{} {}", if compliant { "✅" } else { "❌" }, standard);
    }
    
    // 7. 输出最终建议
    if !security_result.is_secure {
        println!("\n🚨 Immediate Actions Required:");
        println!("============================");
        
        let critical_vulns: Vec<_> = security_result.vulnerabilities.iter()
            .filter(|v| v.severity == Severity::Critical || v.severity == Severity::High)
            .collect();
        
        for (i, vuln) in critical_vulns.iter().enumerate() {
            println!("{}. {} ({}): {}", i + 1, vuln.category, 
                match vuln.severity {
                    Severity::Critical => "Critical",
                    Severity::High => "High",
                    _ => "Medium",
                },
                vuln.description
            );
            println!("   Fix: {}", vuln.recommendation);
        }
        
        println!("\n📞 Consider consulting with security experts if you need assistance with these issues.");
    }
    
    // 8. 返回结果
    if security_result.is_secure {
        println!("\n🎉 Security test completed successfully!");
        println!("📊 Final Security Score: {}/100", security_result.score);
        Ok(())
    } else {
        println!("\n❌ Security test failed!");
        println!("📊 Final Security Score: {}/100", security_result.score);
        println!("🔧 Please address the security issues listed above.");
        Err("Security test failed".into())
    }
}

// 辅助函数
fn has_hardcoded_secrets(result: &SecurityTestResult) -> bool {
    result.vulnerabilities.iter()
        .any(|v| v.category == "Secret Leak")
}

fn has_minimal_permissions(result: &SecurityTestResult) -> bool {
    !result.vulnerabilities.iter()
        .any(|v| v.category == "Permissions" && v.severity == Severity::Medium)
}

fn has_updated_actions(result: &SecurityTestResult) -> bool {
    !result.vulnerabilities.iter()
        .any(|v| v.category == "Dependency Security" && v.severity >= Severity::Medium)
}

fn has_critical_vulnerabilities(result: &SecurityTestResult) -> bool {
    result.vulnerabilities.iter()
        .any(|v| v.severity == Severity::Critical)
}

fn has_high_vulnerabilities(result: &SecurityTestResult) -> bool {
    result.vulnerabilities.iter()
        .any(|v| v.severity == Severity::High)
}