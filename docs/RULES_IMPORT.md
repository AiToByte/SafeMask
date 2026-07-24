# SafeMask 规则导入 / 导出

## 功能概览

| 能力 | 说明 |
|------|------|
| 导入 | 选择本地 `.yaml` / `.yml`（支持多文件）批量导入自定义规则 |
| 导出 | 导出当前全部自定义规则为 YAML |
| 模板 | 下载官方导入模板 |

入口：规则管理页左侧工具栏  
- 上传图标：导入  
- 下载图标：导出  
- 模板图标：下载模板  

## 格式

### RuleGroup（推荐）

```yaml
group: "CUSTOM"
rules:
  - name: "Example_Internal_ID"
    pattern: '\bID-[0-9]{8}\b'
    mask: "<INTERNAL_ID>"
    priority: 20
    enabled: true
```

### 纯数组

```yaml
- name: "Example"
  pattern: "foo"
  mask: "<FOO>"
```

## 字段

| 字段 | 必填 | 说明 |
|------|------|------|
| name | 是 | 全局唯一标识；与内置同名会跳过 |
| pattern | 是 | 正则；会做语法编译校验 |
| mask | 是 | 已有 `<>`/`[]` 则保留；裸标签按当前「脱敏标签包裹样式」自动补齐 |
| priority | 否 | 默认 0，导入时钳制在 -1000~1000 |
| enabled | 否 | 默认 true |

## 冲突策略

- **覆盖自定义**：与已有自定义规则同名 → 覆盖  
- **内置保护**：与内置规则同名 → 跳过，不改内置资源  

## 限制

- 单文件 ≤ 256KB  
- 单次 ≤ 20 个文件  
- 单次 ≤ 500 条规则  
- UTF-8（自动剥离 BOM）  

## 安全与落盘

- 文件由后端读取（dialog 返回路径）  
- 校验通过后写入 `user_rules.yaml.tmp` 再原子替换  
- 成功后热重载引擎  

## 验证命令

```bash
cargo test -p SafeMask rule_import
cargo check -p SafeMask
npm run build
```
