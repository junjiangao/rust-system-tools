# UDisks2 ISO Mounter

一个用于通过UDisks2接口挂载ISO文件的Rust工具，支持命令行和GUI两种模式。

## 功能

- **命令行模式**: 直接挂载和卸载ISO文件
- **GUI模式**: 提供图形界面操作
- **模块化设计**: 易于扩展和维护

## 编译

### 基本版本（仅命令行）
```bash
cargo build --release
```

### 包含GUI功能
```bash
cargo build --release --features gui
```

## 使用方法

### 命令行模式
```bash
# 挂载ISO文件
./target/release/rust-study-examples mount -i /path/to/your.iso

# 或使用长参数
./target/release/rust-study-examples mount --iso-path /path/to/your.iso
```

### GUI模式
```bash
# 启动GUI界面
./target/release/rust-study-examples show-gui
```

## 参数说明

- `-i, --iso-path <FILE>`: 指定ISO文件路径（必需）
- `--show-gui`: 启动GUI模式（可选）

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
└── gui.rs       # GUI界面实现
```

## 开发

项目使用模块化设计，便于扩展：

1. **udisks2模块**: 处理所有UDisks2相关操作
2. **gui模块**: 提供可选的图形界面
3. **main模块**: 应用程序逻辑和CLI解析

## 注意事项

- GUI功能需要编译时启用`gui`特性
- 需要系统安装UDisks2服务
- 某些操作可能需要适当的用户权限