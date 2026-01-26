import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
export const MaskAPI = {
    // 获取规则统计
    async getStats() {
        return await invoke("get_rules_stats");
    },
    // 获取历史记录
    async getHistory() {
        return await invoke("get_mask_history");
    },
    // 切换监控开关
    async toggleMonitor(enabled) {
        await invoke("toggle_monitor", { enabled });
    },
    // 文件脱敏
    async processFile(inputPath, outputPath) {
        return await invoke("process_file_gui", { inputPath, outputPath });
    },
    async getAllRules() {
        return await invoke("get_all_detailed_rules");
    },
    async saveRule(rule) {
        return await invoke("save_rule_api", { rule });
    },
    // 🚀 复制原文并绕过脱敏
    async copyOriginal(text) {
        return await invoke("copy_original_cmd", { text });
    },
    // 🚀 删除规则
    async deleteRule(name) {
        return await invoke("delete_rule_api", { name });
    },
    // 选择文件
    async selectFile() {
        return await open({
            multiple: false,
            filters: [{ name: 'Log/Text', extensions: ['log', 'txt', 'csv', 'json'] }]
        });
    }
};
//# sourceMappingURL=api.js.map