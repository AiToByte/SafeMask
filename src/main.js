/**
 * SafeMask Desktop - 前端入口文件
 * 负责初始化 Vue 3 运行环境、Pinia 状态管理以及全局样式加载
 */
import { createApp } from "vue";
import { createPinia } from "pinia";
// 引入全局样式（包含 Tailwind CSS 指令）
import "./style.css";
// 引入根组件
import App from "./App.vue";
/**
 * 启动应用逻辑
 */
async function bootstrap() {
    // 1. 创建 Vue 应用实例
    const app = createApp(App);
    // 2. 创建并挂载 Pinia 状态管理 (用于跨组件共享脱敏开关、规则数等)
    const pinia = createPinia();
    app.use(pinia);
    // 3. 错误处理机制（全局捕获，增强 App 鲁棒性）
    app.config.errorHandler = (err, instance, info) => {
        console.error("Vue Global Error:", err);
        console.info("Error Info:", info);
        // 此处未来可以集成 Tauri 的日志库，将错误写入本地文件
    };
    // 4. 挂载到 index.html 中的 #app 节点
    app.mount("#app");
}
// 启动
bootstrap();
//# sourceMappingURL=main.js.map