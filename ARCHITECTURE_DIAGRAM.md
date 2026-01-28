# CodeMaster 架构与数据流向图解

CodeMaster 采用 **Local-First (本地优先)** 架构。这意味着没有中央业务服务器，所有业务逻辑和数据存储都在用户自己的设备上完成。

安装包位置:
E:\codemaster\src-tauri\target\release\bundle\msi\CodeMaster_0.1.0_x64_zh-CN.msi

Webview2: 这个应用依赖 Windows 的 Webview2 运行时。Windows 10/11 通常已自带，如果用户无法打开，可能需要安装 Webview2 Runtime (https://developer.microsoft.com/en-us/microsoft-edge/webview2/)。
## 核心架构图

```mermaid
graph TD
    subgraph UserPC ["用户电脑 (Local Environment)"]
        style UserPC fill:#f9f9f9,stroke:#333,stroke-width:2px
        
        User[("👤 用户")]
        
        subgraph App ["CodeMaster 应用程序"]
            style App fill:#e3f2fd,stroke:#2196f3,stroke-width:2px
            
            Frontend["🖥️ 前端界面 (React + Vite)"]
            Backend["⚙️ 后端逻辑 (Rust + Tauri)"]
            
            Frontend <-->|IPC (Tauri Command)| Backend
        end
        
        subgraph Storage ["本地存储"]
            style Storage fill:#fff3e0,stroke:#ff9800,stroke-width:2px
            DB[("🗄️ SQLite 数据库\nsessions.db")]
            Files["📂 用户项目文件\n(源代码)"]
        end
        
        User <-->|交互| Frontend
        Backend <-->|读写| DB
        Backend <-->|读写/执行| Files
    end

    subgraph Cloud ["云端服务 (Cloud Services)"]
        style Cloud fill:#e8f5e9,stroke:#4caf50,stroke-width:2px
        AI_API["☁️ 大模型 API\n(DeepSeek / Qwen)"]
    end

    Backend <-->|HTTPS 请求| AI_API
```

## 简易文本流程图

```
[用户的电脑]
   |
   |-- [ 前端 UI (React) ] <---(展示界面)---> 用户
   |       |
   |   (本地通信)
   |       |
   |-- [ 后端 (Rust) ] <---(读写文件)---> [ 本地数据库 (SQLite) ]
           |
           | (互联网请求 HTTPS)
           |
           v
[ AI 供应商 API (DeepSeek/Qwen) ]
```

## 数据位置说明

### 1. 应用程序在哪？(Server Logic)
CodeMaster 是一个独立的桌面客户端。
- **没有中央服务器**：开发者不需要维护后台服务器。
- **本地运行**：当用户运行 `.exe` / `.msi` 时，Rust 后端进程在用户电脑本地启动，充当了“微型服务器”的角色。

### 2. 数据存在哪？(User Data)
所有聊天记录、会话配置都存储在用户的本地硬盘中。
- **存储路径 (Windows)**: `%LOCALAPPDATA%\codemaster\sessions.db`
  - 通常是: `C:\Users\用户名\AppData\Local\codemaster\sessions.db`
- **隐私安全**：数据完全属于用户，不会上传到开发者的服务器。

### 3. AI 能力来源 (AI Intelligence)
- 应用程序仅作为“连接器”。
- **直接连接**：用户的电脑直接通过互联网连接到 DeepSeek 或通义千问的 API 服务器。
- **API Key**：用户在设置中填写的 API Key 存储在本地系统的安全密钥库（Windows Credential Manager）中。

## 卸载与数据残留

**默认情况下，卸载应用程序不会删除用户数据。**

这是因为数据存储在 `%LOCALAPPDATA%\codemaster` 目录，该目录由应用程序在运行时创建，而非由安装程序创建。标准的 Windows 卸载程序只会删除安装时释放的文件。

- **优点**：用户卸载并重新安装（或更新版本）后，历史聊天记录依然存在。
- **缺点**：如果用户希望彻底清除，需要手动删除 `C:\Users\用户名\AppData\Local\codemaster` 文件夹。

## 优势

1.  **零服务器成本**：开发者分发应用后，无需支付服务器租用费用。
2.  **隐私保护**：除了发给 AI 的内容外，所有操作记录都在本地。
3.  **离线可用性**：除了 AI 对话外，查看历史记录、浏览文件等功能离线可用。
