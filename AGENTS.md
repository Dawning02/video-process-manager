# Repository Guidelines

## 项目结构与模块组织
本仓库是一个 Rust + Slint 跨平台桌面工具，用于搜索并关闭配置好的视频/直播类应用进程。需求来源保留在 `需求文档.md`。源码放在 `src/`：`main.rs` 负责 UI 绑定，`process.rs` 负责进程搜索与关闭，`config.rs` 负责预设应用和自定义配置。Slint 界面文件放在 `ui/`，构建和体积说明放在 `docs/`。

## 构建、测试与开发命令
- `cargo run`：本地运行桌面应用。
- `cargo test`：运行配置校验和进程名匹配等单元测试。
- `cargo build --release`：按仓库 Release 配置构建小体积版本。
- `git diff -- AGENTS.md 需求文档.md Cargo.toml src ui docs`：提交前检查关键改动。

Windows 需要 Rust stable 和 Visual Studio Build Tools 的 Desktop C++ 组件；macOS 需要 Rust stable 和 Xcode Command Line Tools。

## 编码风格与命名约定
除非用户明确要求其他语言，始终使用中文回复用户。Rust 代码使用 2021 edition、4 空格缩进、函数和文件名使用 `snake_case`、类型使用 `PascalCase`。用户可见文案使用中文，集中放在 `ui/main.slint` 或少量状态提示中。避免把进程和配置逻辑写进 UI 回调，优先保持模块可测试。

## 测试指南
对确定性逻辑写 Rust 单元测试，例如空字段校验、重复进程名校验、大小写归一和 `.exe` 后缀兼容。测试名描述行为，例如 `rejects_duplicate_process_names_case_insensitively`。提交前运行 `cargo test`。手动验收需覆盖搜索进程、勾选关闭、未勾选提示、关闭失败提示和自定义应用持久化。

## 提交与 PR 指南
当前仓库尚无正式提交历史。提交信息使用简短祈使句，例如 `Add Rust Slint app skeleton` 或 `Implement custom app config`。PR 应包含变更摘要、验证命令、影响平台、UI 截图，以及进程权限限制或未验证项说明。

## 安全与配置提示
不要提交密钥、机器专属路径、构建产物、`.app`、`.exe` 或临时文件。自定义应用配置应保存在系统用户配置目录，不写入仓库。关闭进程失败时不要静默忽略，应展示类似 `关闭失败：进程名XXX` 的提示。
