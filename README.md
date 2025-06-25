# UDisks2 ISO Mounter

[![Build](https://github.com/junjiangao/rust-study-demo/actions/workflows/quick-build.yml/badge.svg?event=push)](https://github.com/junjiangao/rust-study-demo/actions/workflows/quick-build.yml)

一个用于通过UDisks2接口挂载ISO文件的Rust工具，支持命令行和GUI两种模式。

## 功能

- **命令行模式**: 直接挂载和卸载ISO文件
- **GUI模式**: 提供图形界面操作
- **模块化设计**: 易于扩展和维护

## 编译

### 基本版本
```bash
cargo build
```

## 使用方法

### 命令行模式
```bash
# 挂载ISO文件
./target/release/rust-system-tools mount -i /path/to/your.iso

# 或使用长参数
./target/release/rust-system-tools mount --iso-path /path/to/your.iso
```

### GUI模式
```bash
# 启动GUI界面
./target/release/rust-system-tools show-gui
```

## 参数说明

**命令行模式:**
- `mount -i, --iso-path <FILE>`: 指定ISO文件路径进行挂载

**GUI模式:**
- `show-gui`: 启动图形界面

## 配置文件

程序支持通过配置文件自定义GUI设置：

**配置文件位置:** `~/.config/rust-system-tools/config.toml`

**配置示例:**
```toml
[gui]
# 字体大小和窗口设置
font_size = 14.0
window_width = 600.0
window_height = 450.0

# 智能字体配置
[gui.font_families]
# 中文字体（按优先级）
chinese = [
    "Source Han Sans SC",    # Linux 思源黑体
    "PingFang SC",          # macOS 苹方
    "Microsoft YaHei"       # Windows 微软雅黑
]

# 英文字体（按优先级）
english = [
    "Inter",               # 现代设计字体
    "Segoe UI",           # Windows 系统字体
    "San Francisco"       # macOS 系统字体
]

# 后备字体
fallback = [
    "Noto Sans",
    "Arial",
    "sans-serif"
]
```

**字体配置特性:**
- 🎨 **智能字体系统**: 自动查找系统已安装字体
- 🌍 **多语言支持**: 分别配置中英文字体
- 🔄 **自动降级**: 支持fallback字体链
- 📱 **跨平台**: 支持Linux/macOS/Windows字体名称

首次运行GUI时会自动创建默认配置文件。

## 依赖要求

- Linux系统
- UDisks2服务
- 适当的权限访问系统D-Bus

## 特性

- `gui`: 启用图形界面支持（使用egui框架）

## 项目结构

```
src/
├── main.rs      # 主程序入口和CLI处理
├── lib.rs       # 库接口
├── udisks2.rs   # UDisks2功能实现
├── gui.rs       # GUI界面实现
└── config.rs    # 配置文件实现
```

## 开发

项目使用模块化设计，便于扩展：

1. **udisks2模块**: 处理所有UDisks2相关操作
2. **gui模块**: 提供可选的图形界面
3. **main模块**: 应用程序逻辑和CLI解析
4. **config模块**: 配置文件管理，配合gui使用

## 自动构建

项目使用GitHub Actions自动构建多平台二进制文件：

### 构建流程
- **快速构建**: 每次推送到main分支时触发
- **完整构建**: 创建tag时构建所有平台的发布版本

### 支持平台
- Linux x86_64 (GNU)

### 下载预构建版本
访问[Releases页面](../../releases)下载最新的预构建二进制文件。

## 手动构建

如需本地构建，确保安装了以下依赖：

### Linux
```bash
sudo apt-get install pkg-config libdbus-1-dev libgtk-3-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libssl-dev
```

### 构建命令
```bash
# 版本构建
cargo build --release
```

## 注意事项

- 需要系统安装UDisks2服务
- 某些操作可能需要适当的用户权限