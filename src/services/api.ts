import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

// 🚀 1. 定义 Rule 接口，必须与 Rust 端的 struct 字段名严格一致
export interface Rule {
  name: string;
  pattern: string;
  mask: string;
  priority: number;
  is_custom: boolean; // 🚀 新增标记
}

export interface RuleStats {
  rule_count: number;
  group_count: number;
}

export interface HistoryItem {
  id: string;
  timestamp: string;
  original: string;
  masked: string;
}

export interface AppInfo {
  version: string;
  author: string;
  github: string;
  description: string;
}

export const MaskAPI = {
  // 获取规则统计
  async getStats(): Promise<RuleStats> {
    return await invoke("get_rules_stats");
  },


  // 获取历史记录
  async getHistory(): Promise<HistoryItem[]> {
    return await invoke("get_mask_history");
  },

  // 切换监控开关
  async toggleMonitor(enabled: boolean): Promise<void> {
    await invoke("toggle_monitor", { enabled });
  },

  // 文件脱敏
  async processFile(inputPath: string, outputPath: string): Promise<string> {
    return await invoke("process_file_gui", { inputPath, outputPath });
  },
  async getAllRules(): Promise<Rule[]> {
    return await invoke("get_all_detailed_rules");
  },

  async saveRule(rule: Rule): Promise<string> {
    return await invoke("save_rule_api", { rule });
  },

  // 🚀 复制原文并绕过脱敏
  async copyOriginal(text: string): Promise<void> {
    return await invoke("copy_original_cmd", { text });
  },

  // 🚀 删除规则
  async deleteRule(name: string): Promise<string> {
    return await invoke("delete_rule_api", { name });
  },
  // 选择文件
  async selectFile() {
    return await open({
      multiple: false,
      filters: [{ name: 'Log/Text', extensions: ['log', 'txt', 'csv', 'json'] }]
    });
  },

   // 清除脱敏历史
  async clearHistory(): Promise<void> {
    return await invoke("clear_history_cmd");
  },

  // 获取应用信息
  async getAppInfo(): Promise<AppInfo> {
    return await invoke("get_app_info");
  },

  // 切换窗口置顶状态
  async setAlwaysOnTop(enabled: boolean): Promise<void> {
    await invoke("toggle_always_on_top", { enabled });
  }
  
};