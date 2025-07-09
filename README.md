# Rust ISO 挂载工具 (基于 UDisks2)

[![构建状态](https://github.com/junjiangao/rust-study-demo/actions/workflows/quick-build.yml/badge.svg?event=push)](https://github.com/junjiangao/rust-study-demo/actions/workflows/quick-build.yml)

这是一个仅支持 Linux 平台的 Rust 工具，通过 UDisks2 的 D-Bus 接口实现 ISO 文件的挂载功能。支持命令行和图形界面模式。

## 特性

- 命令行模式：挂载和卸载 ISO 文件
- 图形界面模式：提供操作的图形界面
- 模块化设计，便于维护和扩展

## 构建

### 基础构建
```
cargo build
```

## 使用

### 命令行模式
```
# 挂载 ISO 文件
./target/release/rust-system-tools mount -i /path/to/your.iso

# 或使用长参数
./target/release/rust-system-tools mount --iso-path /path/to/your.iso
```

### 图形界面模式
```
# 启动图形界面
./target/release/rust-system-tools show-gui
```

## 参数

**命令行模式：**
- `mount -i, --iso-path <FILE>` : 指定要挂载的 ISO 文件路径

**图形界面模式：**
- `show-gui` : 启动图形界面

## 配置

图形界面配置文件路径：

`~/.config/rust-system-tools/config.toml`

示例配置内容：
```toml
[gui]
font_size = 14.0
window_width = 600.0
window_height = 450.0

[gui.font_families]
chinese = [
    "Source Han Sans SC",
    "PingFang SC",
    "Microsoft YaHei"
]
english = [
    "Inter",
    "Segoe UI",
    "San Francisco"
]
fallback = [
    "Noto Sans",
    "Arial",
    "sans-serif"
]
```

## 依赖

- Linux 系统
- UDisks2 服务
- 访问系统 D-Bus 的相应权限

## 特性

- `gui`：启用图形界面支持（底层使用 egui 框架）

## 项目结构

```
src/
├── main.rs      # 入口和命令行处理
├── lib.rs       # 库接口
├── udisks2.rs   # UDisks2 相关功能实现
├── gui.rs       # 图形界面实现
└── config.rs    # 配置文件实现
```

## 开发说明

模块划分：

1. udisks2 模块：处理所有 UDisks2 操作
2. gui 模块：提供图形界面（可选）
3. main 模块：应用逻辑和命令行解析
4. config 模块：配置管理，被 GUI 使用

## 自动化构建

使用 GitHub Actions 支持多平台二进制构建：

### 构建流程
- 快速构建：每次 push main 分支触发
- 完整构建：打 tag 时构建全部平台

### 支持平台
- Linux x86_64 (GNU)

### 预编译版本下载

在 [Releases 页面](../../releases) 下载最新的预编译包。

## 手动安装依赖

Linux:
```bash
sudo apt-get install pkg-config libdbus-1-dev libgtk-3-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libssl-dev
```

## 构建命令

```bash
# 构建发布版本
cargo build --release
```

## 备注

- 需要系统安装 UDisks2 服务
- 某些操作需要相应用户权限
