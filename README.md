# CodeMaster

> AI Coding Agent for Chinese Developers | 面向中国开发者的 AI 编码助手

[English](#english) | [中文](#中文)

---

## English

### Overview

CodeMaster is an open-source AI coding agent desktop application designed specifically for Chinese developers. It integrates with Chinese LLM providers (DeepSeek, Qwen, etc.) to provide a seamless coding assistance experience without requiring VPN access.

### Features

- **Native LLM Integration**: Supports Chinese LLM providers (DeepSeek API)
- **Agent Tools**: File operations, code search, bash execution, LSP integration
- **Embedded Terminal**: Built-in terminal with xterm.js
- **Session Management**: Local session storage with SQLite
- **Bilingual UI**: Full Chinese and English interface support

### Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **Database**: SQLite
- **LLM**: DeepSeek API (OpenAI-compatible)

### Installation

Download the latest `.msi` installer from [Releases](https://github.com/xxx/codemaster/releases).

### Development

```bash
# Install dependencies
npm install

# Start development server
npm run tauri dev

# Build for production
npm run tauri build
```

### License

MIT License - see [LICENSE](LICENSE) for details.

---

## 中文

### 概述

CodeMaster 是一个开源的 AI 编码代理桌面应用，专为中国开发者设计。它集成了国产大模型（DeepSeek、通义千问等），无需翻墙即可获得流畅的 AI 编码辅助体验。

### 特性

- **原生大模型集成**：支持国产大模型（DeepSeek API）
- **代理工具**：文件操作、代码搜索、bash 执行、LSP 集成
- **内嵌终端**：基于 xterm.js 的内置终端
- **会话管理**：基于 SQLite 的本地会话存储
- **双语界面**：完整的中英文界面支持

### 技术栈

- **前端**: React + TypeScript + Vite
- **后端**: Rust + Tauri v2
- **数据库**: SQLite
- **大模型**: DeepSeek API（OpenAI 兼容）

### 安装

从 [Releases](https://github.com/xxx/codemaster/releases) 下载最新的 `.msi` 安装包。

### 开发

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### 许可证

MIT License - 详见 [LICENSE](LICENSE)。

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解贡献指南。
