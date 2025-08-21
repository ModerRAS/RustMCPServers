use std::io::Write;
use github_actions_tests::security_tester::SecurityTester;

fn main() {
    println!("🔍 Debugging secret detection...");
    
    // 创建测试文件
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "api_key: 'sk-1234567890abcdef1234567890abcdef'").unwrap();
    writeln!(temp_file, "secret: 'super_secret_value_1234567890abcdef'").unwrap();
    writeln!(temp_file, "password: 'mypassword123'").unwrap();
    writeln!(temp_file, "token: 'github_token_1234567890abcdef1234567890'").unwrap();
    
    // 输出文件内容
    let path = temp_file.path().to_str().unwrap();
    println!("📄 File path: {}", path);
    println!("📄 File content:");
    let content = std::fs::read_to_string(path).unwrap();
    println!("{}", content);
    
    // 运行测试
    let tester = SecurityTester::new(path).unwrap();
    let result = tester.run_security_tests();
    
    println!("📊 Test results:");
    println!("  - Total vulnerabilities: {}", result.vulnerabilities.len());
    println!("  - Security score: {}", result.score);
    println!("  - Is secure: {}", result.is_secure);
    
    for (i, vuln) in result.vulnerabilities.iter().enumerate() {
        println!("  {}. {}: {} - {}", i + 1, vuln.category, vuln.title, vuln.description);
    }
    
    // 检查是否有Secret Leak类型的漏洞
    let secret_leaks: Vec<_> = result.vulnerabilities.iter()
        .filter(|v| v.category == "Secret Leak")
        .collect();
    
    println!("🔑 Secret leaks found: {}", secret_leaks.len());
    for leak in secret_leaks {
        println!("  - {}: {}", leak.title, leak.location);
    }
}