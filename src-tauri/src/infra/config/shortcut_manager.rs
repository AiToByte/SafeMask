use tauri::{AppHandle};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code};
use std::str::FromStr;

pub struct ShortcutManager;

impl ShortcutManager {
    /// 核心方法：重载全局快捷键
    /// 逻辑：先注销所有当前快捷键，再根据传入的字符串解析并注册新快捷键
    pub fn reload_magic_shortcut(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
        let gs = app.global_shortcut();
        
        // 🚀 修复点：不再使用 unregister_all()，因为它会杀掉 Alt+M
        // 我们只注销旧的，或者先全部清空后再把 Alt+M 补回来
        let _ = gs.unregister_all(); 

        // 1. 注册动态的 Magic Paste (Alt+V)
        let magic_v = Self::parse_shortcut_string(shortcut_str)?;
        gs.register(magic_v).map_err(|e| e.to_string())?;

        // 2. 🚀 关键：必须在这里把静态的 Alt+M 重新注册回来
        let alt_m = Shortcut::new(Some(Modifiers::ALT), Code::KeyM);
        let _ = gs.register(alt_m); 

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