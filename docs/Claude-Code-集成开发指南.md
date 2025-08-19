# Claude Code 集成开发指南

## 概述

Claude Code 是 Anthropic 官方的命令行工具，可以让开发者通过命令行与 Claude AI 进行交互。本指南详细介绍了如何在你的 Agent 项目中集成 Claude Code，实现强大的 AI 编程助手功能。

## 目录

1. [快速开始](#快速开始)
2. [核心架构](#核心架构)
3. [配置方式](#配置方式)
4. [调用流程](#调用流程)
5. [消息处理](#消息处理)
6. [上下文管理](#上下文管理)
7. [错误处理](#错误处理)
8. [完整实现示例](#完整实现示例)
9. [最佳实践](#最佳实践)
10. [常见问题](#常见问题)

## 快速开始

### 1. 安装 Claude Code

```bash
# 通过 npm 安装
npm install -g @anthropic-ai/claude-code

# 或通过官方安装脚本
curl -fsSL https://claude.ai/install.sh | sh
```

### 2. 基本调用

```bash
# 最简单的调用方式
claude -p "Hello, how are you?"

# 指定模型
claude -p "Write a hello world function" --model claude-sonnet-4-20250514
```

### 3. 在 Node.js 中调用

```typescript
import { execa } from 'execa'

async function callClaudeCode(prompt: string, messages: any[] = []) {
    const args = [
        '-p',
        '--verbose',
        '--output-format', 'stream-json',
        '--disallowedTools', 'Task,Bash,Glob,Grep,LS,Read,Edit,MultiEdit,Write',
        '--max-turns', '1'
    ]
    
    const child = execa('claude', args, {
        stdin: 'pipe',
        stdout: 'pipe',
        stderr: 'pipe',
        timeout: 600000 // 10分钟超时
    })
    
    // 发送消息
    const stdinData = JSON.stringify(messages)
    child.stdin.write(stdinData)
    child.stdin.end()
    
    // 处理响应
    for await (const line of readline.createInterface({ input: child.stdout })) {
        if (line.trim()) {
            const chunk = JSON.parse(line)
            console.log('Received:', chunk)
        }
    }
}
```

## 核心架构

### 组件设计

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Your Agent   │    │  Claude Code    │    │   Claude API    │
│                 │    │      CLI        │    │                 │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │Message      │ │    │ │Process      │ │    │ │AI Model     │ │
│ │Handler      │ │◄──►│ │Manager      │ │◄──►│ │             │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │Context      │ │    │ │Stream       │ │    │ │Session      │ │
│ │Manager      │ │    │ │Parser       │ │    │ │Manager      │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 关键组件说明

1. **Message Handler** - 负责与 Claude Code CLI 通信
2. **Context Manager** - 管理对话历史和上下文
3. **Process Manager** - 处理子进程的创建和通信
4. **Stream Parser** - 解析 Claude Code 返回的流式响应

## 配置方式

### 1. 基本配置

```typescript
interface ClaudeCodeConfig {
    // Claude Code CLI 路径 (可选，默认为 'claude')
    claudePath?: string
    
    // 使用的模型
    modelId?: string
    
    // 最大输出令牌数
    maxOutputTokens?: number
    
    // 工作目录
    cwd?: string
    
    // 超时时间 (毫秒)
    timeout?: number
    
    // 是否使用 Vertex AI
    useVertex?: boolean
}
```

### 2. 环境变量

```bash
# 使用 Vertex AI
export CLAUDE_CODE_USE_VERTEX=1

# 注意：CLAUDE_CODE_MAX_OUTPUT_TOKENS 不是环境变量
# 需要通过配置参数传递
```

### 3. 支持的模型

```typescript
const SUPPORTED_MODELS = {
    "claude-sonnet-4-20250514": {
        maxTokens: 200000,
        supportsImages: false,
        supportsPromptCache: true
    },
    "claude-opus-4-1-20250805": {
        maxTokens: 200000,
        supportsImages: false,
        supportsPromptCache: true
    },
    "claude-opus-4-20250514": {
        maxTokens: 200000,
        supportsImages: false,
        supportsPromptCache: true
    },
    "claude-3-7-sonnet-20250219": {
        maxTokens: 200000,
        supportsImages: false,
        supportsPromptCache: true
    },
    "claude-3-5-sonnet-20241022": {
        maxTokens: 200000,
        supportsImages: false,
        supportsPromptCache: true
    },
    "claude-3-5-haiku-20241022": {
        maxTokens: 200000,
        supportsImages: false,
        supportsPromptCache: true
    }
}
```

### 4. Vertex AI 模型转换

```typescript
function convertModelNameForVertex(modelName: string): string {
    // 将 claude-sonnet-4-20250514 转换为 claude-sonnet-4@20250514
    return modelName.replace(/-(\d{8})$/, '@$1')
}
```

## 调用流程

### 1. 完整调用流程

```typescript
async function completeClaudeCodeFlow(
    systemPrompt: string,
    messages: Array<{role: string, content: any}>,
    config: ClaudeCodeConfig
) {
    // 1. 过滤消息
    const filteredMessages = filterMessagesForClaudeCode(messages)
    
    // 2. 构建参数
    const args = buildCommandArgs(systemPrompt, config)
    
    // 3. 启动进程
    const process = await startClaudeProcess(args, config)
    
    // 4. 发送数据
    await sendDataToProcess(process, filteredMessages, config)
    
    // 5. 处理响应
    const results = await processStreamResponse(process)
    
    // 6. 清理资源
    await cleanupProcess(process)
    
    return results
}
```

### 2. 命令行参数构建

```typescript
function buildCommandArgs(systemPrompt: string, config: ClaudeCodeConfig): string[] {
    const isWindows = process.platform === 'win32'
    const args = ['-p']
    
    // 非Windows平台通过命令行传递系统提示
    if (!isWindows) {
        args.push('--system-prompt', systemPrompt)
    }
    
    // 通用参数
    args.push(
        '--verbose',
        '--output-format', 'stream-json',
        '--disallowedTools', getDisallowedTools(),
        '--max-turns', '1'
    )
    
    // 模型参数
    if (config.modelId) {
        const modelId = config.useVertex 
            ? convertModelNameForVertex(config.modelId)
            : config.modelId
        args.push('--model', modelId)
    }
    
    return args
}
```

### 3. 平台差异处理

```typescript
function getStdinData(
    systemPrompt: string,
    messages: any[],
    isWindows: boolean
): string {
    if (isWindows) {
        // Windows平台：系统提示+消息都通过stdin传递
        return JSON.stringify({
            systemPrompt,
            messages
        })
    } else {
        // 其他平台：只传递消息
        return JSON.stringify(messages)
    }
}
```

## 消息处理

### 1. 消息过滤

```typescript
function filterMessagesForClaudeCode(messages: any[]): any[] {
    return messages.map((message) => {
        if (typeof message.content === 'string') {
            return message
        }
        
        const filteredContent = message.content.map((block: any) => {
            if (block.type === 'image') {
                // 将图像转换为文本描述
                return {
                    type: 'text',
                    text: `[Image (${block.source?.type}): ${block.source?.media_type} not supported]`
                }
            }
            return block
        })
        
        return { ...message, content: filteredContent }
    })
}
```

### 2. 响应类型定义

```typescript
type ClaudeCodeMessage = 
    | InitMessage
    | AssistantMessage
    | ErrorMessage
    | ResultMessage

interface InitMessage {
    type: 'system'
    subtype: 'init'
    session_id: string
    tools: string[]
    mcp_servers: string[]
    apiKeySource: 'none' | '/login managed key' | string
}

interface AssistantMessage {
    type: 'assistant'
    message: {
        id: string
        type: 'message'
        role: 'assistant'
        content: Array<{
            type: 'text' | 'thinking' | 'redacted_thinking' | 'tool_use'
            text?: string
            thinking?: string
        }>
        usage: {
            input_tokens: number
            output_tokens: number
            cache_read_input_tokens?: number
            cache_creation_input_tokens?: number
        }
        stop_reason: string | null
    }
    session_id: string
}

interface ResultMessage {
    type: 'result'
    result: any
    total_cost_usd: number
}
```

### 3. 响应处理

```typescript
async function* processStreamResponse(process: any): AsyncGenerator<any> {
    const rl = readline.createInterface({ input: process.stdout })
    let partialData = ''
    
    for await (const line of rl) {
        if (!line.trim()) continue
        
        try {
            // 尝试解析当前行
            const chunk = JSON.parse(line)
            
            // 处理不同类型的消息
            switch (chunk.type) {
                case 'system':
                    if (chunk.subtype === 'init') {
                        yield { type: 'init', data: chunk }
                    }
                    break
                    
                case 'assistant':
                    for (const content of chunk.message.content) {
                        switch (content.type) {
                            case 'text':
                                yield { type: 'text', text: content.text }
                                break
                            case 'thinking':
                                yield { type: 'reasoning', text: content.thinking || '' }
                                break
                            case 'redacted_thinking':
                                yield { type: 'reasoning', text: '[Redacted thinking block]' }
                                break
                            case 'tool_use':
                                console.warn('tool_use is not supported')
                                break
                        }
                    }
                    break
                    
                case 'result':
                    yield { type: 'result', data: chunk }
                    break
            }
        } catch (error) {
            // 如果解析失败，累积数据
            partialData += line
            try {
                const chunk = JSON.parse(partialData)
                yield chunk
                partialData = ''
            } catch {
                // 继续累积
            }
        }
    }
}
```

## 上下文管理

### 1. 上下文存储结构

```typescript
interface ConversationMessage {
    role: 'user' | 'assistant'
    content: any
    timestamp?: number
}

interface ContextManager {
    messages: ConversationMessage[]
    maxTokens: number
    systemPrompt: string
    lastSummary?: {
        content: string
        timestamp: number
        tokenCount: number
    }
}
```

### 2. 上下文管理实现

```typescript
class ContextManager {
    private messages: ConversationMessage[] = []
    private maxTokens: number = 100000
    private systemPrompt: string
    
    constructor(systemPrompt: string, maxTokens: number = 100000) {
        this.systemPrompt = systemPrompt
        this.maxTokens = maxTokens
    }
    
    // 添加消息
    addMessage(role: 'user' | 'assistant', content: any): void {
        this.messages.push({
            role,
            content,
            timestamp: Date.now()
        })
        
        // 检查是否需要压缩上下文
        this.checkContextLength()
    }
    
    // 获取用于发送的消息
    getMessagesForSending(): ConversationMessage[] {
        // 获取自上次摘要以来的消息
        const messagesSinceLastSummary = this.getMessagesSinceLastSummary()
        return this.filterMessages(messagesSinceLastSummary)
    }
    
    // 获取自上次摘要以来的消息
    private getMessagesSinceLastSummary(): ConversationMessage[] {
        // 这里可以实现更复杂的逻辑
        return this.messages
    }
    
    // 过滤消息
    private filterMessages(messages: ConversationMessage[]): ConversationMessage[] {
        return messages.map(msg => ({
            ...msg,
            content: this.filterMessageContent(msg.content)
        }))
    }
    
    // 过滤消息内容
    private filterMessageContent(content: any): any {
        if (typeof content === 'string') {
            return content
        }
        
        if (Array.isArray(content)) {
            return content.map(block => {
                if (block.type === 'image') {
                    return {
                        type: 'text',
                        text: `[Image not supported]`
                    }
                }
                return block
            })
        }
        
        return content
    }
    
    // 检查上下文长度
    private checkContextLength(): void {
        const estimatedTokens = this.estimateTokenCount()
        
        if (estimatedTokens > this.maxTokens) {
            this.compressContext()
        }
    }
    
    // 估算token数量
    private estimateTokenCount(): number {
        // 简单的token估算
        const text = this.messages.map(msg => 
            typeof msg.content === 'string' ? msg.content : JSON.stringify(msg.content)
        ).join(' ')
        
        return Math.ceil(text.length / 4) // 粗略估算
    }
    
    // 压缩上下文
    private async compressContext(): Promise<void> {
        // 这里可以实现上下文压缩逻辑
        // 例如：创建摘要、删除旧消息等
        console.log('Context length exceeds limit, consider implementing compression')
    }
}
```

### 3. 完整的上下文管理示例

```typescript
class ClaudeCodeAgent {
    private contextManager: ContextManager
    private config: ClaudeCodeConfig
    
    constructor(systemPrompt: string, config: ClaudeCodeConfig) {
        this.contextManager = new ContextManager(systemPrompt, config.maxOutputTokens || 16000)
        this.config = config
    }
    
    async sendMessage(userMessage: string): Promise<string> {
        // 添加用户消息到上下文
        this.contextManager.addMessage('user', userMessage)
        
        // 获取要发送的消息
        const messages = this.contextManager.getMessagesForSending()
        
        // 调用 Claude Code
        const response = await this.callClaudeCode(messages)
        
        // 添加助手回复到上下文
        this.contextManager.addMessage('assistant', response)
        
        return response
    }
    
    private async callClaudeCode(messages: ConversationMessage[]): Promise<string> {
        // 实现调用逻辑
        const args = buildCommandArgs(this.contextManager['systemPrompt'], this.config)
        const process = await startClaudeProcess(args, this.config)
        
        const stdinData = getStdinData(
            this.contextManager['systemPrompt'],
            messages,
            process.platform === 'win32'
        )
        
        // 发送数据并处理响应
        // ... 实现细节
        return 'Response from Claude Code'
    }
}
```

## 错误处理

### 1. 常见错误类型

```typescript
class ClaudeCodeError extends Error {
    constructor(
        message: string,
        public type: 'not_found' | 'timeout' | 'api_error' | 'process_error',
        public originalError?: Error
    ) {
        super(message)
        this.name = 'ClaudeCodeError'
    }
}

class ClaudeCodeNotFoundError extends ClaudeCodeError {
    constructor(claudePath: string, originalError: Error) {
        super(
            `Claude Code not found at path: ${claudePath}. Please install it from https://docs.anthropic.com/en/docs/claude-code/setup`,
            'not_found',
            originalError
        )
        this.name = 'ClaudeCodeNotFoundError'
    }
}

class ClaudeCodeTimeoutError extends ClaudeCodeError {
    constructor(timeout: number) {
        super(
            `Claude Code process timed out after ${timeout}ms`,
            'timeout'
        )
        this.name = 'ClaudeCodeTimeoutError'
    }
}
```

### 2. 错误处理实现

```typescript
async function safeClaudeCodeCall(
    systemPrompt: string,
    messages: any[],
    config: ClaudeCodeConfig
): Promise<any> {
    try {
        const process = await startClaudeProcess(args, config)
        
        // 处理进程错误
        process.on('error', (error) => {
            if (error.code === 'ENOENT') {
                throw new ClaudeCodeNotFoundError(config.claudePath || 'claude', error)
            }
            throw new ClaudeCodeError(`Process error: ${error.message}`, 'process_error', error)
        })
        
        // 处理超时
        const timeout = setTimeout(() => {
            process.kill()
            throw new ClaudeCodeTimeoutError(config.timeout || 600000)
        }, config.timeout || 600000)
        
        try {
            const results = await processStreamResponse(process)
            clearTimeout(timeout)
            return results
        } catch (error) {
            clearTimeout(timeout)
            throw error
        }
        
    } catch (error) {
        if (error instanceof ClaudeCodeError) {
            throw error
        }
        
        // 处理 API 错误
        if (error.message.includes('API Error')) {
            const errorMatch = error.message.match(/\{.*\}/)
            if (errorMatch) {
                try {
                    const errorData = JSON.parse(errorMatch[0])
                    if (errorData.error?.message?.includes('Invalid model name')) {
                        throw new ClaudeCodeError(
                            `Invalid model name: ${config.modelId}`,
                            'api_error',
                            error
                        )
                    }
                } catch {
                    // 忽略 JSON 解析错误
                }
            }
        }
        
        throw new ClaudeCodeError(
            `Unexpected error: ${error.message}`,
            'process_error',
            error
        )
    }
}
```

### 3. 进程清理

```typescript
async function cleanupProcess(process: any): Promise<void> {
    try {
        if (!process.killed) {
            process.kill()
        }
        
        // 等待进程完全退出
        await new Promise((resolve) => {
            process.on('exit', resolve)
            setTimeout(resolve, 5000) // 5秒超时
        })
    } catch (error) {
        console.warn('Error cleaning up process:', error)
    }
}
```

## 完整实现示例

### 1. 完整的 Claude Code 集成类

```typescript
import { execa } from 'execa'
import readline from 'readline'
import * as os from 'os'

export class ClaudeCodeIntegration {
    private config: ClaudeCodeConfig
    private contextManager: ContextManager
    
    constructor(systemPrompt: string, config: ClaudeCodeConfig = {}) {
        this.config = {
            claudePath: 'claude',
            modelId: 'claude-sonnet-4-20250514',
            maxOutputTokens: 16000,
            timeout: 600000,
            cwd: process.cwd(),
            useVertex: false,
            ...config
        }
        
        this.contextManager = new ContextManager(systemPrompt, this.config.maxOutputTokens)
    }
    
    async sendMessage(userMessage: string): Promise<string> {
        // 添加用户消息
        this.contextManager.addMessage('user', userMessage)
        
        // 获取要发送的消息
        const messages = this.contextManager.getMessagesForSending()
        
        // 调用 Claude Code
        const response = await this.callClaudeCode(messages)
        
        // 添加助手回复
        this.contextManager.addMessage('assistant', response)
        
        return response
    }
    
    async sendMessageStream(userMessage: string): Promise<AsyncGenerator<any>> {
        this.contextManager.addMessage('user', userMessage)
        const messages = this.contextManager.getMessagesForSending()
        
        return this.callClaudeCodeStream(messages)
    }
    
    private async callClaudeCode(messages: any[]): Promise<string> {
        const args = this.buildArgs()
        const process = await this.startProcess(args)
        
        try {
            const response = await this.processResponse(process, messages)
            return response
        } finally {
            await this.cleanupProcess(process)
        }
    }
    
    private async *callClaudeCodeStream(messages: any[]): AsyncGenerator<any> {
        const args = this.buildArgs()
        const process = await this.startProcess(args)
        
        try {
            for await (const chunk of this.processResponseStream(process, messages)) {
                yield chunk
            }
        } finally {
            await this.cleanupProcess(process)
        }
    }
    
    private buildArgs(): string[] {
        const isWindows = os.platform() === 'win32'
        const args = ['-p']
        
        if (!isWindows) {
            args.push('--system-prompt', this.contextManager['systemPrompt'])
        }
        
        args.push(
            '--verbose',
            '--output-format', 'stream-json',
            '--disallowedTools', this.getDisallowedTools(),
            '--max-turns', '1'
        )
        
        if (this.config.modelId) {
            const modelId = this.config.useVertex
                ? this.convertModelNameForVertex(this.config.modelId)
                : this.config.modelId
            args.push('--model', modelId)
        }
        
        return args
    }
    
    private async startProcess(args: string[]): Promise<any> {
        try {
            return execa(this.config.claudePath!, args, {
                stdin: 'pipe',
                stdout: 'pipe',
                stderr: 'pipe',
                cwd: this.config.cwd,
                timeout: this.config.timeout,
                env: {
                    ...process.env,
                    CLAUDE_CODE_MAX_OUTPUT_TOKENS: this.config.maxOutputTokens?.toString()
                }
            })
        } catch (error: any) {
            if (error.code === 'ENOENT') {
                throw new ClaudeCodeNotFoundError(this.config.claudePath!, error)
            }
            throw error
        }
    }
    
    private async processResponse(process: any, messages: any[]): Promise<string> {
        const isWindows = os.platform() === 'win32'
        const stdinData = isWindows
            ? JSON.stringify({
                systemPrompt: this.contextManager['systemPrompt'],
                messages
            })
            : JSON.stringify(messages)
        
        // 发送数据
        process.stdin.write(stdinData)
        process.stdin.end()
        
        // 收集响应
        let responseText = ''
        
        for await (const chunk of this.processResponseStream(process, messages)) {
            if (chunk.type === 'text') {
                responseText += chunk.text
            }
        }
        
        return responseText
    }
    
    private async *processResponseStream(process: any, messages: any[]): AsyncGenerator<any> {
        const isWindows = os.platform() === 'win32'
        const stdinData = isWindows
            ? JSON.stringify({
                systemPrompt: this.contextManager['systemPrompt'],
                messages
            })
            : JSON.stringify(messages)
        
        // 发送数据
        setImmediate(() => {
            try {
                process.stdin.write(stdinData)
                process.stdin.end()
            } catch (error) {
                console.error('Error writing to stdin:', error)
            }
        })
        
        // 处理响应流
        const rl = readline.createInterface({ input: process.stdout })
        let partialData = ''
        
        for await (const line of rl) {
            if (!line.trim()) continue
            
            try {
                const chunk = JSON.parse(line)
                
                switch (chunk.type) {
                    case 'system':
                        if (chunk.subtype === 'init') {
                            yield { type: 'init', data: chunk }
                        }
                        break
                        
                    case 'assistant':
                        for (const content of chunk.message.content) {
                            switch (content.type) {
                                case 'text':
                                    yield { type: 'text', text: content.text }
                                    break
                                case 'thinking':
                                    yield { type: 'reasoning', text: content.thinking || '' }
                                    break
                                case 'redacted_thinking':
                                    yield { type: 'reasoning', text: '[Redacted thinking block]' }
                                    break
                                case 'tool_use':
                                    console.warn('tool_use is not supported')
                                    break
                            }
                        }
                        break
                        
                    case 'result':
                        yield { type: 'result', data: chunk }
                        break
                }
            } catch (error) {
                partialData += line
                try {
                    const chunk = JSON.parse(partialData)
                    yield chunk
                    partialData = ''
                } catch {
                    // 继续累积
                }
            }
        }
    }
    
    private async cleanupProcess(process: any): Promise<void> {
        try {
            if (!process.killed) {
                process.kill()
            }
            
            await new Promise((resolve) => {
                process.on('exit', resolve)
                setTimeout(resolve, 5000)
            })
        } catch (error) {
            console.warn('Error cleaning up process:', error)
        }
    }
    
    private getDisallowedTools(): string {
        return [
            'Task', 'Bash', 'Glob', 'Grep', 'LS', 'exit_plan_mode',
            'Read', 'Edit', 'MultiEdit', 'Write',
            'NotebookRead', 'NotebookEdit', 'WebFetch',
            'TodoRead', 'TodoWrite', 'WebSearch'
        ].join(',')
    }
    
    private convertModelNameForVertex(modelName: string): string {
        return modelName.replace(/-(\d{8})$/, '@$1')
    }
}
```

### 2. 使用示例

```typescript
// 基本使用
const agent = new ClaudeCodeIntegration('You are a helpful programming assistant')

// 发送消息
const response = await agent.sendMessage('Write a hello world function in JavaScript')
console.log(response)

// 流式响应
for await (const chunk of agent.sendMessageStream('Explain this code')) {
    if (chunk.type === 'text') {
        process.stdout.write(chunk.text)
    }
}
```

### 3. 高级配置示例

```typescript
const agent = new ClaudeCodeIntegration(
    'You are an expert code reviewer',
    {
        modelId: 'claude-opus-4-20250514',
        maxOutputTokens: 32000,
        timeout: 300000,
        useVertex: true,
        claudePath: '/usr/local/bin/claude'
    }
)

// 带上下文的对话
await agent.sendMessage('Please review this code: function add(a, b) { return a + b }')
await agent.sendMessage('What are the potential issues with this code?')
```

## 最佳实践

### 1. 性能优化

```typescript
// 1. 使用连接池管理进程
class ProcessPool {
    private pool: any[] = []
    private maxPoolSize: number = 3
    
    async getProcess(): Promise<any> {
        if (this.pool.length > 0) {
            return this.pool.pop()
        }
        
        if (this.pool.length < this.maxPoolSize) {
            return await this.createProcess()
        }
        
        // 等待可用进程
        return new Promise((resolve) => {
            const checkPool = () => {
                if (this.pool.length > 0) {
                    resolve(this.pool.pop())
                } else {
                    setTimeout(checkPool, 100)
                }
            }
            checkPool()
        })
    }
    
    returnProcess(process: any): void {
        if (!process.killed) {
            this.pool.push(process)
        }
    }
}
```

### 2. 错误重试

```typescript
class RetryableClaudeCode extends ClaudeCodeIntegration {
    async sendMessageWithRetry(
        userMessage: string,
        maxRetries: number = 3
    ): Promise<string> {
        let lastError: Error
        
        for (let attempt = 1; attempt <= maxRetries; attempt++) {
            try {
                return await this.sendMessage(userMessage)
            } catch (error) {
                lastError = error as Error
                
                // 如果是致命错误，立即重试
                if (error instanceof ClaudeCodeNotFoundError) {
                    throw error
                }
                
                // 指数退避
                const delay = Math.pow(2, attempt) * 1000
                await new Promise(resolve => setTimeout(resolve, delay))
            }
        }
        
        throw lastError!
    }
}
```

### 3. 缓存机制

```typescript
interface CacheEntry {
    key: string
    response: string
    timestamp: number
    ttl: number
}

class CachedClaudeCode extends ClaudeCodeIntegration {
    private cache: Map<string, CacheEntry> = new Map()
    private maxCacheSize: number = 100
    
    private generateCacheKey(messages: any[]): string {
        const content = messages.map(msg => 
            `${msg.role}:${JSON.stringify(msg.content)}`
        ).join('|')
        
        return require('crypto')
            .createHash('md5')
            .update(content)
            .digest('hex')
    }
    
    async sendMessageWithCache(userMessage: string): Promise<string> {
        const messages = this.contextManager.getMessagesForSending()
        const cacheKey = this.generateCacheKey(messages)
        
        // 检查缓存
        const cached = this.cache.get(cacheKey)
        if (cached && Date.now() - cached.timestamp < cached.ttl) {
            return cached.response
        }
        
        // 调用 API
        const response = await this.sendMessage(userMessage)
        
        // 缓存结果
        this.cache.set(cacheKey, {
            key: cacheKey,
            response,
            timestamp: Date.now(),
            ttl: 300000 // 5分钟
        })
        
        // 清理过期缓存
        this.cleanupCache()
        
        return response
    }
    
    private cleanupCache(): void {
        const now = Date.now()
        for (const [key, entry] of this.cache.entries()) {
            if (now - entry.timestamp > entry.ttl) {
                this.cache.delete(key)
            }
        }
        
        // 如果缓存太大，删除最老的条目
        if (this.cache.size > this.maxCacheSize) {
            const entries = Array.from(this.cache.entries())
            entries.sort((a, b) => a[1].timestamp - b[1].timestamp)
            
            const toDelete = entries.slice(0, entries.length - this.maxCacheSize)
            toDelete.forEach(([key]) => this.cache.delete(key))
        }
    }
}
```

### 4. 监控和日志

```typescript
class MonitoredClaudeCode extends ClaudeCodeIntegration {
    private metrics = {
        totalRequests: 0,
        successfulRequests: 0,
        failedRequests: 0,
        totalTokens: 0,
        averageResponseTime: 0
    }
    
    async sendMessage(userMessage: string): Promise<string> {
        const startTime = Date.now()
        this.metrics.totalRequests++
        
        try {
            const response = await super.sendMessage(userMessage)
            
            this.metrics.successfulRequests++
            const responseTime = Date.now() - startTime
            this.updateAverageResponseTime(responseTime)
            
            this.logRequest('success', userMessage, responseTime)
            return response
        } catch (error) {
            this.metrics.failedRequests++
            const responseTime = Date.now() - startTime
            this.logRequest('error', userMessage, responseTime, error)
            throw error
        }
    }
    
    private updateAverageResponseTime(responseTime: number): void {
        const total = this.metrics.successfulRequests + this.metrics.failedRequests
        this.metrics.averageResponseTime = 
            (this.metrics.averageResponseTime * (total - 1) + responseTime) / total
    }
    
    private logRequest(
        status: 'success' | 'error',
        message: string,
        responseTime: number,
        error?: Error
    ): void {
        const logEntry = {
            timestamp: new Date().toISOString(),
            status,
            messageLength: message.length,
            responseTime,
            error: error?.message
        }
        
        console.log('Claude Code Request:', JSON.stringify(logEntry, null, 2))
    }
    
    getMetrics() {
        return { ...this.metrics }
    }
}
```

## 常见问题

### 1. Claude Code 未找到

**问题**：`Error: Claude Code not found at path: claude`

**解决方案**：
```bash
# 检查是否已安装
claude --version

# 如果未安装，请按照官方文档安装
curl -fsSL https://claude.ai/install.sh | sh

# 或者检查 PATH 环境变量
which claude
```

### 2. 模型不匹配错误

**问题**：`Invalid model name` 或 `Model not found`

**解决方案**：
```typescript
// 检查模型名称是否正确
const validModels = [
    'claude-sonnet-4-20250514',
    'claude-opus-4-20250514',
    'claude-3-5-sonnet-20241022'
]

if (!validModels.includes(config.modelId)) {
    throw new Error(`Invalid model: ${config.modelId}`)
}
```

### 3. 进程超时

**问题**：`Claude Code process timed out`

**解决方案**：
```typescript
// 增加超时时间
const config = {
    timeout: 1200000 // 20分钟
}

// 或者减少消息复杂度
const shortMessages = messages.slice(-5) // 只保留最近5条消息
```

### 4. 内存不足

**问题**：进程因为内存不足而被杀死

**解决方案**：
```typescript
// 限制上下文长度
const maxMessages = 20
const truncatedMessages = messages.slice(-maxMessages)

// 或者实现上下文压缩
const compressedMessages = await compressContext(messages)
```

### 5. 平台特定问题

**Windows 平台**：
```typescript
// Windows 有命令行长度限制，确保使用 stdin 传递数据
const isWindows = process.platform === 'win32'
if (isWindows) {
    // 确保系统提示和消息都通过 stdin 传递
    const stdinData = JSON.stringify({
        systemPrompt,
        messages
    })
}
```

**Linux/macOS 平台**：
```typescript
// Linux 有 execve 参数长度限制
const isLinux = process.platform === 'linux'
if (isLinux) {
    // 确保命令行参数不要太长
    const maxArgLength = 128 * 1024 // 128KB
    if (JSON.stringify(messages).length > maxArgLength) {
        // 使用 stdin 传递消息
    }
}
```

## 总结

本指南提供了完整的 Claude Code 集成方案，包括：

1. **核心架构设计** - 理解 Claude Code 的工作原理
2. **完整的实现代码** - 可以直接使用的集成类
3. **上下文管理** - 如何维护对话历史
4. **错误处理** - 处理各种异常情况
5. **性能优化** - 缓存、重试、连接池等
6. **最佳实践** - 监控、日志、调试技巧

通过本指南，你可以在自己的 Agent 项目中快速集成 Claude Code，提供强大的 AI 编程助手功能。

## 资源链接

- [Claude Code 官方文档](https://docs.anthropic.com/en/docs/claude-code/setup)
- [Anthropic API 文档](https://docs.anthropic.com/en/api/messages)
- [Node.js execa 文档](https://github.com/sindresorhus/execa)
- [Roo Code 项目](https://github.com/RooCodeInc/Roo-Code)

---

*文档版本：1.0*  
*最后更新：2025-08-18*  
*基于 Roo Code 项目实践编写*