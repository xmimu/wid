# 前端文件结构

重新组织后的前端文件结构更加清晰和规范。

## 目录结构

```
src/
├── index.html              # 主HTML文件
├── assets/                 # 静态资源文件
│   ├── javascript.svg
│   └── tauri.svg
├── css/                    # 样式文件
│   └── bootstrap.min.css   # Bootstrap CSS框架
├── js/                     # JavaScript源代码
│   ├── main.js            # 主应用逻辑
│   ├── waapi.js           # WAAPI辅助函数
│   └── waapi-query.js     # WAAPI查询模块
└── lib/                    # 第三方库文件
    ├── autobahn.min.js    # Autobahn.js WAMP客户端
    └── bootstrap.bundle.min.js # Bootstrap JS框架
```

## 文件分类说明

### `css/` - 样式文件
- 存放所有CSS样式文件
- 目前包含Bootstrap CSS框架
- 未来可以添加自定义样式文件

### `js/` - JavaScript源代码
- 存放项目自己的JavaScript源代码
- `main.js`: 主应用逻辑和UI交互
- `waapi.js`: WAAPI相关辅助函数
- `waapi-query.js`: WAAPI查询功能模块

### `lib/` - 第三方库
- 存放外部JavaScript库和框架
- `autobahn.min.js`: WAMP协议客户端，用于WAAPI连接
- `bootstrap.bundle.min.js`: Bootstrap JavaScript组件

### `assets/` - 静态资源
- 存放图片、图标等静态资源文件
- 目前包含Tauri和JavaScript的SVG图标

## 变更记录

### 已删除的文件
- `styles.css` - 原有的空样式文件已删除

### 文件移动
- 所有JavaScript源文件移至 `js/` 目录
- 第三方库文件移至 `lib/` 目录  
- CSS文件移至 `css/` 目录
- 更新了 `index.html` 中的文件引用路径

## 优势

1. **清晰的目录结构**: 按文件类型和功能分类
2. **易于维护**: 文件组织更加规范
3. **便于扩展**: 新文件可以很容易地放到对应目录
4. **符合规范**: 遵循前端项目的标准目录结构