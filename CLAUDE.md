# CLAUDE.md

本文件为 Rust ISO 挂载工具项目的 Claude Code 使用指南。

## 基本使用命令

### 构建
- `cargo build`：基础构建
- `cargo build --release`：发布构建

### 运行
- 命令行挂载模式：
```bash
./target/release/rust-system-tools mount -i /path/to/your.iso
```
- 图形界面模式：
```bash
./target/release/rust-system-tools show-gui
```
### 代码检查
- 格式检查：`cargo fmt -- --check`
- 静态分析：`cargo clippy -- -D warnings`

### 测试
- 使用命令： `cargo test`

## 架构概述

本项目是基于 Rust 的 Linux 平台 ISO 挂载工具，使用系统的 UDisks2 D-Bus 接口实现挂载功能。

关键模块如下：
- `src/main.rs`：程序入口和命令行参数解析
- `src/lib.rs`：库接口
- `src/udisks2.rs`：与 UDisks2 服务交互核心代码
- `src/gui.rs`：图形界面实现（可选特性）
- `src/config.rs`：配置文件读写，实现 GUI 设置持久化
- `src/wim.rs`：WIM 文件解析器，支持读取 Windows ISO 镜像的版本信息

## 新功能特性

### WIM 文件解析器
- **智能版本检测**：自动识别 Windows ISO 镜像中的版本信息
- **多镜像支持**：可以解析包含多个版本的 WIM 文件（如 Pro、Home、Enterprise）
- **架构识别**：自动检测系统架构（x86、x64、ARM64）
- **完整元数据**：提取镜像名称、描述、文件数量、总大小等详细信息
- **回退机制**：如果 WIM 解析失败，自动回退到文本文件分析方法

### 支持的文件格式
- `install.wim`：标准 Windows 安装镜像
- `install.esd`：压缩 Windows 安装镜像
- 自动检测并解析 XML 元数据

## 运行环境

- 仅支持 Linux（基于系统 D-Bus 和 UDisks2 特殊接口，非跨平台）

## 重要提示

- 项目模块划分逻辑清晰，便于维护和扩展
- GUI 为可选特性，默认编译不带 GUI
- 使用前需确保系统安装 UDisks2 服务，并有权限访问 D-Bus
- 新增的 WIM 解析功能大大提升了对 Windows ISO 镜像的支持

## 行为准则
- 每次任务，如果涉及到代码的修改，文件应用修改完成后，执行代码检查
