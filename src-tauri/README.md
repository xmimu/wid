# Rust 模块结构

本项目将原来的单一 `lib.rs` 文件按功能拆分为多个模块，提高代码的可维护性和组织性。

## 模块结构

```
src/
├── lib.rs                      # 主入口文件
└── modules/
    ├── mod.rs                  # 模块索引文件
    ├── types.rs                # 数据类型定义
    ├── utils.rs                # 工具函数
    ├── validators.rs           # 目录验证器
    ├── wwise_search.rs         # Wwise工程文件搜索
    └── bank_search.rs          # Bank目录搜索
```

## 各模块职责

### `types.rs`
- 定义共用的数据结构
- `SearchResult` - 搜索结果结构体

### `utils.rs`
- 通用工具函数
- `is_valid_guid()` - GUID格式验证

### `validators.rs`
- 目录验证相关函数
- `validate_wwise_directory()` - 验证Wwise工程目录
- `validate_bank_directory()` - 验证Bank目录

### `wwise_search.rs`
- Wwise工程文件搜索功能
- `search_wwise_project()` - 在.wwu文件中搜索ID

### `bank_search.rs`
- Bank目录搜索功能
- `search_bank_directory()` - 在SoundbanksInfo文件中搜索ID
- `parse_soundbanks_json()` - JSON格式解析
- `parse_soundbanks_xml()` - XML格式解析

## 模块依赖

- 所有模块通过 `modules/mod.rs` 统一导出
- `lib.rs` 只需要导入需要的函数
- 各模块间通过明确的接口进行通信
- 减少了代码重复和依赖混乱

## 优势

1. **模块化**: 按功能分离，便于维护
2. **可测试性**: 每个模块可以独立测试
3. **代码复用**: 工具函数可以在多个模块间共享
4. **清晰度**: 代码结构更清晰，易于理解
5. **扩展性**: 新功能可以很容易地添加为新模块