# 测试说明

本目录包含了项目的集成测试。

## 测试文件

- `wim_parser_test.rs` - WIM文件解析器的集成测试

## 运行测试

### 运行集成测试

```bash
cargo test
```

### 运行特定的测试文件

```bash
cargo test --test wim_parser_test
```

### 运行特定的测试函数

```bash
cargo test --test wim_parser_test test_parse_arch_from_xml
```

## 测试内容

### WIM解析器测试

- `test_parse_arch_from_xml` - 测试从XML中解析架构信息
- `test_parse_single_image_xml_with_arch` - 测试单个镜像XML解析
- `test_different_arch_values` - 测试不同架构值的解析
- `test_version_extraction` - 测试版本信息提取
- `test_architecture_priority` - 测试架构信息优先级（XML优先于名称推断）
- `test_fallback_architecture_detection` - 测试回退架构检测机制

## 架构值映射

测试验证了以下架构值的正确映射：

- `0` → `x86` (32位)
- `9` → `x64` (64位)
- `5` → `ARM` (32位)
- `12` → `ARM64` (64位)

## 注意事项

- 测试使用 `/dev/null` 作为虚拟文件，不需要实际的WIM文件
- 测试覆盖了架构解析的各种场景，包括优先级和回退机制
- 集成测试可以直接运行，无需额外特性