# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Structure

This is a Rust workspace.

## Quick Commands

### Development
```bash
# Navigate to server directory

# Build and run
cargo run

# Run tests
cargo test

# Run specific test
cargo test test_search_request_building -- --nocapture

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build release
cargo build --release
```

- Reference Crates: https://github.com/modelcontextprotocol/rust-sdk.git，你可以Clone这个仓库到tmp文件夹里自行查找文档，这是Rust官方的MCP SDK工具。
- 如果你需要开发新的MCP服务器，请在servers文件夹内建立新的Rust项目来开发。
- 开发时编写的项目相关文档放在对应Rust项目的docs文件夹内
- 开发时如果需要编写全局可用的文档，放在仓库根目录的docs文件夹内
- 每个Rust项目的开发进度放在对应Rust项目中的CLAUDE.md中，更新进度的时候更新在对应目录的，注意不要改错文件
- 仓库根目录的CLAUDE.md文件请不要随意更改，如果你需要更改，可以添加到CLAUDE.local.md中
## 关于 Github Actions
- 如果需要新建发布版本用的github actions，可以参考docs/monorepo-release-guide.md
## 注意事项
- 根目录下的tmp文件夹是用来放一些参考用的临时文件的，这个文件夹是被gitignore掉的，所以如果你有需要参考的git仓库，或者是需要参考的示例文件之类的你可以clone或者下载到tmp文件夹里，然后用来参考。
