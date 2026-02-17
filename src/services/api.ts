import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

// ─────────────────────────────────────────────────────────────────────────────
// 1. 类型定义 (需与 Rust struct 严格对应)
// ─────────────────────────────────────────────────────────────────────────────

/** 脱敏规则条目 */
export interface Rule {
  name: string;
  pattern: string;
  mask: string;
  priority: number;
  is_custom: boolean;
  enabled: boolean;
}

/** 历史记录条目 (包含宇宙模式标记) */
export interface HistoryItem {
  id: string;
  timestamp: string;
  original: string;
  masked: string;
  mode: 'SHADOW' | 'SENTRY'; // 标记该记录产生的环境
}

/** 全局应用设置 (影子模式核心配置) */
export interface AppSettings {
  magic_paste_shortcut: string;
  shadow_mode_enabled: boolean;
  paste_delay_ms: number;
  enable_visual_feedback: boolean;
  enable_audio_feedback: boolean;
}

/** 规则库统计 */
export interface RuleStats {
  rule_count: number;
}

/** 文件处理响应 */
export interface ProcessResponse {
  message: string;
  output_path: string;
  output_dir: string;
  duration: string;
  throughput: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. API 对象实现
// ─────────────────────────────────────────────────────────────────────────────

export const MaskAPI = {
  
  // === 设置与系统管理 ===

  /** 获取持久化设置 */
  async getSettings(): Promise<AppSettings> {
    return await invoke("get_app_settings");
  },

  /** 更新并持久化设置 (触发快捷键热重载) */
  async updateSettings(settings: AppSettings): Promise<string> {
    return await invoke("update_app_settings", { newSettings: settings });
  },

  /** 切换宇宙模式 (Alt+M 逻辑) */
  async toggleVaultMode(): Promise<boolean> {
    return await invoke("toggle_vault_mode");
  },

  // === 规则管理与测试沙盒 ===

  /** 获取所有详细规则列表 */
  async getAllRules(): Promise<Rule[]> {
    return await invoke("get_all_detailed_rules");
  },

  /** 注入并保存新规则 */
  async saveRule(rule: Rule): Promise<string> {
    return await invoke("save_rule_api", { rule });
  },

  /** 移除自定义规则 */
  async deleteRule(name: string): Promise<string> {
    return await invoke("delete_rule_api", { name });
  },

  /** 
   * [核心] 规则实时测试沙盒 
   * 调用 Rust 临时编译正则并返回脱敏预览
   */
  async testRule(pattern: string, mask: string, testText: string): Promise<string> {
    return await invoke("test_rule_logic", { pattern, mask, testText });
  },

  /** 获取规则条数统计 */
  async getStats(): Promise<RuleStats> {
    return await invoke("get_rules_stats");
  },

  // === 剪贴板历史审计 ===

  /** 获取审计历史记录 */
  async getHistory(): Promise<HistoryItem[]> {
    return await invoke("get_mask_history");
  },

  /** 销毁所有历史痕迹 */
  async clearHistory(): Promise<void> {
    return await invoke("clear_history_cmd");
  },

  /** 特殊复制：将原文写入剪贴板并绕过脱敏监听器 */
  async copyOriginal(text: string): Promise<void> {
    return await invoke("copy_original_cmd", { text });
  },

  // === 大规模文件处理 ===

  /** 启动多核文件脱敏流水线 */
  async processFile(inputPath: string): Promise<ProcessResponse> {
    return await invoke("process_file_gui", { inputPath });
  },

  /** 打开文件所在目录 */
  async openFolder(path: string): Promise<void> {
    const { revealItemInDir } = await import('@tauri-apps/plugin-opener');
    await revealItemInDir(path);
  },

  /** 唤起系统文件选择框 */
  async selectFile() {
    return await open({
      multiple: false,
      filters: [{ name: 'Log/Text/Office', extensions: ['log', 'txt', 'csv', 'json', 'docx', 'xlsx', 'pdf'] }]
    });
  },
  async getAppInfo() { return await invoke("get_app_info"); },
  /** 切换窗口置顶状态 */
  async setAlwaysOnTop(enabled: boolean): Promise<void> {
    await invoke("toggle_always_on_top", { enabled });
  },
  async setRecordingMode(enabled: boolean): Promise<void> {
  await invoke("set_recording_mode", { enabled });
}
};