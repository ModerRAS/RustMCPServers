module.exports = {
  apps: [{
    name: 'simple-task-orchestrator',
    script: 'cargo',
    args: 'run --bin mcp_server',
    cwd: '/root/WorkSpace/Rust/RustMCPServers/servers/simple-task-orchestrator',
    interpreter: 'none',
    watch: false,
    autorestart: true,
    max_memory_restart: '1G',
    env: {
      RUST_LOG: 'info',
      RUST_BACKTRACE: '1'
    },
    merge_logs: true,
    error_file: '/tmp/simple-task-orchestrator-error.log',
    out_file: '/tmp/simple-task-orchestrator-out.log',
    log_file: '/tmp/simple-task-orchestrator-combined.log',
    time: true
  }]
}