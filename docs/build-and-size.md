# 构建环境与体积预估

## 开发环境

- Rust stable toolchain：通过 `rustup` 安装。
- Windows：安装 Visual Studio Build Tools，并勾选 Desktop development with C++。
- macOS：安装 Xcode Command Line Tools：`xcode-select --install`。

## 常用命令

```powershell
cargo run
cargo check
cargo test
cargo build --release
```

当前项目已通过 `cargo check` 和 `cargo test`。如果访问 crates.io 失败，仓库内 `.cargo/config.toml` 已配置 `rsproxy.cn` 镜像。

## 运行依赖

项目使用 Rust + Slint 构建原生桌面应用，不依赖 Electron、Chromium、Node.js、Python 或 Qt 运行时。

主要 Cargo 依赖：

- `slint`：跨平台桌面 UI。
- `sysinfo`：枚举并关闭进程。
- `serde`、`toml`：读写自定义应用配置。
- `directories`：定位系统用户配置目录。
- `anyhow`：错误处理。

## 自定义配置文件

自定义应用保存为程序所在目录下的 `config.toml`，建议与 `video-process-manager.exe`、`presets.toml` 放在同一文件夹。旧版本保存在用户配置目录下的 `config.toml` 仍可兼容读取，但新的保存位置固定为程序所在目录。配置示例：

```toml
[[custom_apps]]
app_name = "测试应用"
process_names = ["notepad.exe", "helper.exe"]
```

`config.toml` 只保存用户自定义应用，不保存预设应用。

## 预设配置文件

`presets.toml` 保存视频应用预设进程名。程序启动时优先读取用户配置目录下的 `presets.toml`，如果不存在则读取程序当前目录/仓库根目录的 `presets.toml`；如果文件缺失、为空或格式错误，会回退到代码内置预设并在状态栏提示。配置示例：

```toml
[[preset_apps]]
app_name = "芒果TV"
process_names = ["MangoTV.exe", "mgtv.exe"]
```

后续如需联网更新，可下载同格式 TOML 到用户配置目录并替换 `presets.toml`，不需要改动用户的 `config.toml`。

## Release 体积优化

`Cargo.toml` 已配置发布优化：

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## 体积预估

- Windows `.exe`
  - Debug：约 30-80 MB，不适合分发。
  - Release 默认：约 6-15 MB。
  - Release + 当前体积优化：约 3-10 MB。
- macOS `.app`
  - 单架构 Release：约 5-15 MB。
  - Universal Binary：约 10-25 MB。

实际体积会受 Slint 渲染后端、图标/图片/字体资源、签名元数据和目标架构影响。当前项目使用软件渲染后端，并避免大型资源和 WebView。

## Windows 分发流程

推荐先生成绿色版，再封装安装包。

绿色版：

```powershell
.\scripts\build-portable.ps1
```

输出：

```text
dist/VideoProcessManager/
  video-process-manager.exe
  presets.toml
  config.toml
```

安装包：

```powershell
iscc .\installer\video-process-manager.iss
```

输出：

```text
dist/VideoProcessManagerSetup-0.1.0.exe
```

安装包依赖 Inno Setup。当前未配置代码签名，安装时可能显示“未知发布者”。
