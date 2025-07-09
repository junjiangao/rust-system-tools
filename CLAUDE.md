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

## 运行环境

- 仅支持 Linux（基于系统 D-Bus 和 UDisks2 特殊接口，非跨平台）

## 重要提示

- 项目模块划分逻辑清晰，便于维护和扩展
- GUI 为可选特性，默认编译不带 GUI
- 使用前需确保系统安装 UDisks2 服务，并有权限访问 D-Bus

## 行为准则
- 每次修改代码文件后，使用`cargo fmt`格式化代码
