use std::path::Path;
use std::fs;

/// 验证 Wwise 工程目录（必须包含 .wproj 文件）
#[tauri::command]
pub fn validate_wwise_directory(path: String) -> Result<bool, String> {
    let dir_path = Path::new(&path);
    
    if !dir_path.exists() {
        return Err("目录不存在".to_string());
    }
    
    if !dir_path.is_dir() {
        return Err("路径不是目录".to_string());
    }
    
    // 检查目录中是否有 .wproj 文件
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "wproj" {
                        return Ok(true);
                    }
                }
            }
            Err("目录中未找到 .wproj 文件".to_string())
        }
        Err(e) => Err(format!("无法读取目录: {}", e)),
    }
}

/// 验证 Bank 目录（必须包含 SoundbanksInfo.xml 或 SoundbanksInfo.json）
#[tauri::command]
pub fn validate_bank_directory(path: String) -> Result<bool, String> {
    let dir_path = Path::new(&path);
    
    if !dir_path.exists() {
        return Err("目录不存在".to_string());
    }
    
    if !dir_path.is_dir() {
        return Err("路径不是目录".to_string());
    }
    
    // 检查是否存在 SoundbanksInfo.xml 或 SoundbanksInfo.json
    let xml_path = dir_path.join("SoundbanksInfo.xml");
    let json_path = dir_path.join("SoundbanksInfo.json");
    
    if xml_path.exists() || json_path.exists() {
        Ok(true)
    } else {
        Err("目录中未找到 SoundbanksInfo.xml 或 SoundbanksInfo.json".to_string())
    }
}