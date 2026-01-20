import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface RuleStats {
  rule_count: number;
  group_count: number;
}

export const MaskAPI = {
  // 获取规则统计
  async getStats(): Promise<RuleStats> {
    return await invoke("get_rules_stats");
  },

  // 切换监控开关
  async toggleMonitor(enabled: boolean): Promise<void> {
    await invoke("toggle_monitor", { enabled });
  },

  // 文件脱敏
  async processFile(inputPath: string, outputPath: string): Promise<string> {
    return await invoke("process_file_gui", { inputPath, outputPath });
  },

  // 选择文件
  async selectFile() {
    return await open({
      multiple: false,
      filters: [{ name: 'Log/Text', extensions: ['log', 'txt', 'csv', 'json'] }]
    });
  }
};