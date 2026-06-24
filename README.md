# 视频应用进程管理工具

一个基于 Rust + Slint 的轻量桌面工具，用于搜索并关闭指定的视频/直播类应用进程。当前主要面向 Windows PC，代码结构也为后续 macOS 支持预留了空间。

## 功能特性

- 预设常见视频应用：腾讯视频、爱奇艺、芒果TV、优酷、哔哩哔哩、抖音。
- 支持自定义应用和多个进程名，例如 `notepad.exe, helper.exe`。
- 搜索结果按应用分组展示，支持展开查看子进程。
- 任务管理器式列表展示：名称、类型、PID、CPU、内存、状态。
- 应用窗口支持拖拽调整大小和系统最大化，搜索结果和自定义应用列表会随窗口伸缩。
- 支持单选、应用分组批量选择、全选、反选和一键关闭。
- Windows 下普通关闭失败时，会在校验 PID + 进程名后使用 `taskkill /PID /T /F` 兜底。
- 预设配置和自定义配置均使用 TOML，方便手动维护和后续联网更新。

## 环境要求

- Rust stable toolchain。
- Windows：Visual Studio Build Tools，并安装 Desktop development with C++。
- macOS：Xcode Command Line Tools，可通过 `xcode-select --install` 安装。

本项目不依赖 Electron、Chromium、Node.js、Python 或 Qt 运行时。

## 开发命令

```powershell
cargo run
cargo check
cargo test
cargo build --release
```

- `cargo run`：本地启动桌面应用。
- `cargo check`：快速检查 Rust 与 Slint 绑定是否可编译。
- `cargo test`：运行配置解析、进程名匹配和搜索结果分组等单元测试。
- `cargo build --release`：构建发布版本。

如果访问 crates.io 失败，仓库内 `.cargo/config.toml` 已配置 `rsproxy.cn` 镜像。

## 配置文件

### 自定义应用

自定义应用保存在程序所在目录的 `config.toml`，建议和 `video-process-manager.exe`、`presets.toml` 放在同一文件夹。程序内可通过“打开配置文件”访问。

```toml
[[custom_apps]]
app_name = "测试应用"
process_names = ["notepad.exe", "helper.exe"]
```

### 预设应用

仓库根目录的 `presets.toml` 保存默认预设。程序启动时按以下顺序加载：

1. 用户配置目录下的 `presets.toml`
2. 程序当前目录或仓库根目录的 `presets.toml`
3. 代码内置预设

```toml
[[preset_apps]]
app_name = "芒果TV"
process_names = ["MangoTV.exe", "mgtv.exe"]
```

后续如果加入联网更新，可下载同格式 TOML 到用户配置目录并替换 `presets.toml`。

### 分发目录建议

```text
视频应用进程管理工具/
  video-process-manager.exe
  presets.toml
  config.toml
```

`config.toml` 可以没有，首次添加自定义应用时程序会自动创建。旧版本保存在用户配置目录的 `config.toml` 仍可被兼容读取，但新的保存位置固定为程序所在目录。

## 打包分发

### 绿色版

绿色版适合先验证程序在没有 Rust 环境的电脑上能否直接运行：

```powershell
.\scripts\build-portable.ps1
```

输出目录：

```text
dist/VideoProcessManager/
  video-process-manager.exe
  presets.toml
  config.toml
```

将整个 `VideoProcessManager` 文件夹复制到其他电脑，双击 `video-process-manager.exe` 即可运行。

### Inno Setup 安装包

安装 Inno Setup 后，先生成绿色版，再编译安装脚本：

```powershell
.\scripts\build-portable.ps1
iscc .\installer\video-process-manager.iss
```

输出文件：

```text
dist/VideoProcessManagerSetup-0.1.0.exe
```

安装包默认安装到：

```text
%LOCALAPPDATA%\VideoProcessManager
```

这样普通用户可以写入同目录的 `config.toml`。当前未做代码签名，Windows 可能提示“未知发布者”。

## 项目结构

```text
src/
  main.rs       Slint UI 绑定、状态管理和交互回调
  config.rs     TOML 配置读写、预设应用、自定义应用校验
  process.rs    进程搜索、CPU/内存采样和关闭逻辑
ui/
  main.slint    桌面界面定义
docs/
  build-and-size.md           构建环境与体积预估
  verification-fix-list.md    验证、修复和后续计划
presets.toml    预设应用进程配置
需求文档.md      原始需求说明
```

## 当前验证状态

当前代码已通过：

```powershell
cargo check
cargo test
```

仍建议在 Windows 上执行手动验收：

- 启动 `cargo run`，确认窗口可打开。
- 添加 `测试应用 / notepad.exe`，搜索、展开、勾选并关闭。
- 添加或确认 `mgtv.exe`，复测芒果TV一键关闭。
- 修改 `presets.toml` 或 `config.toml` 后重新加载/重启确认生效。

## 发布体积预估

`Cargo.toml` 已配置 Release 体积优化：

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

预估体积：

- Windows Release：约 3-10 MB。
- macOS 单架构 `.app`：约 5-15 MB。
- macOS Universal `.app`：约 10-25 MB。

实际体积会受目标平台、签名、图标、资源文件和 Slint 渲染后端影响。
