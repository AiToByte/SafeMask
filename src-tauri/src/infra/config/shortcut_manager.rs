use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code};
use std::str::FromStr;
use log::{info, error};

pub struct ShortcutManager;

impl ShortcutManager {
    /// 核心方法：重载全局快捷键
    /// 逻辑：先注销所有当前快捷键，再根据传入的字符串解析并注册新快捷键
    pub fn reload_magic_shortcut(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
        let gs = app.global_shortcut();
        
        // 1. 安全起见，先注销所有已由本插件注册的快捷键
        // 注：在生产环境建议记录具体的快捷键对象进行精确注销，这里使用 unregister_all 简化重载流程
        let _ = gs.unregister_all();

        // 2. 解析字符串 (例如 "Alt+Shift+V")
        let shortcut = Self::parse_shortcut_string(shortcut_str)
            .map_err(|e| {
                error!("快捷键解析失败: {}", e);
                e
            })?;

        // 3. 执行注册
        gs.register(shortcut).map_err(|e| {
            let err_msg = format!("快捷键 [{}] 被占用或注册失败: {}", shortcut_str, e);
            error!("{}", err_msg);
            err_msg
        })?;
        
        info!("🚀 影子模式快捷键已成功注册: {}", shortcut_str);
        Ok(())
    }

    /// 字符串解析器：将 "Alt+Shift+V" 映射为 Tauri 核心类型
    pub fn parse_shortcut_string(input: &str) -> Result<Shortcut, String> {
        let parts: Vec<&str> = input.split('+').collect();
        if parts.is_empty() {
            return Err("快捷键格式不能为空".into());
        }

        let mut modifiers = Modifiers::empty();
        let mut key_code: Option<Code> = None;

        for part in parts {
            let clean_part = part.trim().to_uppercase();
            match clean_part.as_str() {
                // 修饰键判定
                "ALT" => modifiers.insert(Modifiers::ALT),
                "CTRL" | "CONTROL" => modifiers.insert(Modifiers::CONTROL),
                "SHIFT" => modifiers.insert(Modifiers::SHIFT),
                "SUPER" | "COMMAND" | "META" | "WIN" => modifiers.insert(Modifiers::SUPER),
                
                // 主键判定 (例如 "V", "M", "P", "ENTER")
                key_str => {
                    // 构造 Tauri 内部使用的 KeyCode 格式，例如 "KeyV"
                    let formatted_key = if key_str.len() == 1 {
                        format!("Key{}", key_str)
                    } else {
                        // 处理特殊键，如 "Enter", "Space" (需符合 Code 枚举名)
                        key_str.to_string()
                    };

                    if let Ok(code) = Code::from_str(&formatted_key) {
                        key_code = Some(code);
                    } else if let Ok(code) = Code::from_str(key_str) {
                        key_code = Some(code);
                    }
                }
            }
        }

        match key_code {
            Some(code) => Ok(Shortcut::new(Some(modifiers), code)),
            None => Err(format!("无法识别主按键: {}", input)),
        }
    }
}