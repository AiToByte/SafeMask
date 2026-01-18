const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

document.addEventListener("DOMContentLoaded", async () => {
  const switchEl = document.getElementById("monitor-switch");
  const dropZone = document.getElementById("drop-zone");
  const idleView = document.getElementById("idle-view");
  const progressView = document.getElementById("progress-view");
  const progressBar = document.getElementById("progress-bar");
  const progressPercent = document.getElementById("progress-percent");

  // 1. 监听监控开关
  switchEl.addEventListener("change", async (e) => {
    await invoke("toggle_monitor", { enabled: e.target.checked });
  });

  // 2. 监听脱敏事件通知
  listen("masked-event", (event) => {
    // 实际项目中建议使用 Toast 库
    console.log("SafeMask:", event.payload);
  });

  // 3. 监听大文件进度
  listen("file-progress", (event) => {
    idleView.classList.add("hidden");
    progressView.classList.remove("hidden");
    const { percentage } = event.payload;
    progressBar.style.width = `${percentage}%`;
    progressPercent.innerText = `${Math.round(percentage)}%`;
  });

  // 4. 处理拖拽文件
  listen("tauri://file-drop", async (event) => {
    const filePath = event.payload[0];
    if (!filePath) return;

    dropZone.classList.add("drag-active");
    const outPath = filePath.replace(/\.[^/.]+$/, "") + ".masked.log";
    
    try {
      const msg = await invoke("process_file_gui", { 
        inputPath: filePath, 
        outputPath: outPath 
      });
      alert(msg);
      // 处理完重置界面
      idleView.classList.remove("hidden");
      progressView.classList.add("hidden");
    } catch (e) {
      alert("错误: " + e);
    } finally {
      dropZone.classList.remove("drag-active");
    }
  });
});
