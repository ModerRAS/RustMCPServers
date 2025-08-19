#!/bin/bash

echo "🔍 Testing regex patterns..."
cd /root/WorkSpace/Rust/RustMCPServers/tests

# 创建测试文件
cat > test_regex.txt << 'EOF'
api_key: 'sk-1234567890abcdef1234567890abcdef'
secret: 'super_secret_value_1234567890abcdef'
password: 'mypassword123'
token: 'github_token_1234567890abcdef1234567890'
EOF

echo "📄 Test content:"
cat test_regex.txt

# 创建一个简单的Rust程序来测试正则表达式
cat > test_regex.rs << 'EOF'
use regex::Regex;

fn main() {
    let test_line = "api_key: 'sk-1234567890abcdef1234567890abcdef'";
    
    let patterns = vec![
        r#"api[_-]?key\s*[:=]\s*["'][A-Za-z0-9+/]{20,}["']"#,
        r#"secret\s*[:=]\s*["'][A-Za-z0-9+/]{20,}["']"#,
        r#"password\s*[:=]\s*["'][A-Za-z0-9+/]{8,}["']"#,
        r#"token\s*[:=]\s*["'][A-Za-z0-9+/]{20,}["']"#,
    ];
    
    println!("🔍 Testing line: {}", test_line);
    
    for (i, pattern) in patterns.iter().enumerate() {
        println!("Pattern {}: {}", i + 1, pattern);
        let re = Regex::new(pattern).unwrap();
        if re.is_match(test_line) {
            println!("  ✅ MATCH!");
        } else {
            println!("  ❌ No match");
        }
    }
}
EOF

echo ""
echo "🔍 Compiling and running regex test..."
rustc --edition 2021 test_regex.rs --extern regex=../target/debug/deps/libregex-*.rlib -o test_regex 2>/dev/null

if [ -f test_regex ]; then
    ./test_regex
    rm -f test_regex
else
    echo "❌ Failed to compile regex test"
fi

# 清理
rm -f test_regex.txt test_regex.rs