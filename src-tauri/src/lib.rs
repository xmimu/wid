use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use glob::glob;
use rayon::prelude::*;
use roxmltree::Document;

// 搜索结果结构
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub id: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 验证 Wwise 工程目录（必须包含 .wproj 文件）
#[tauri::command]
fn validate_wwise_directory(path: String) -> Result<bool, String> {
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
fn validate_bank_directory(path: String) -> Result<bool, String> {
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

/// 在 Wwise 工程文件中搜索 ID
/// 
/// # 参数
/// * `directory` - Wwise 工程目录路径（包含 .wproj 文件）
/// * `id_string` - 要搜索的 ID 字符串
/// * `id_types` - 要搜索的 ID 类型数组，可选值: ["GUID", "ShortID", "MediaID"]
/// 
/// # 返回
/// * `Ok(Vec<SearchResult>)` - 搜索结果列表
/// * `Err(String)` - 搜索失败，返回错误信息
#[tauri::command]
fn search_wwise_project(
    directory: String,
    id_string: String,
    id_types: Vec<String>,
) -> Result<Vec<SearchResult>, String> {
    // 验证目录
    let dir_path = Path::new(&directory);
    if !dir_path.is_dir() {
        return Err("目录不存在".to_string());
    }
    
    // 查询字符串转小写用于匹配
    let query = id_string.to_lowercase();
    
    // 使用 glob 查找所有 .wwu 文件
    let pattern = format!("{}/**/*.wwu", directory);
    let entries: Vec<_> = glob(&pattern)
        .map_err(|e| format!("Glob 模式错误: {}", e))?
        .filter_map(Result::ok)
        .collect();
    
    if entries.is_empty() {
        return Err("未找到 .wwu 文件".to_string());
    }
    
    // 使用并行处理所有文件
    let results: Vec<SearchResult> = entries
        .par_iter()
        .flat_map(|file_path| {
            // 读取文件内容
            let contents = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => return Vec::new(),
            };
            
            // 解析 XML
            let doc = match Document::parse(&contents) {
                Ok(d) => d,
                Err(_) => return Vec::new(),
            };
            
            let mut file_results = Vec::new();
            
            // 根据 id_types 执行不同的搜索
            for id_type in &id_types {
                match id_type.as_str() {
                    "GUID" => {
                        // 搜索 GUID (ID 属性)
                        for node in doc.descendants().filter(|n| n.has_attribute("ID")) {
                            let id = node.attribute("ID").unwrap_or("");
                            if id.to_lowercase().contains(&query) {
                                let name = node.attribute("Name").unwrap_or("未命名");
                                file_results.push(SearchResult {
                                    name: format!("{} ({})", name, node.tag_name().name()),
                                    id: id.to_string(),
                                });
                            }
                        }
                    }
                    "ShortID" => {
                        // 搜索 ShortID 属性
                        for node in doc.descendants().filter(|n| n.has_attribute("ShortID")) {
                            let short_id = node.attribute("ShortID").unwrap_or("");
                            if short_id.to_lowercase().contains(&query) {
                                let name = node.attribute("Name").unwrap_or("未命名");
                                let id = node.attribute("ID").unwrap_or(short_id);
                                file_results.push(SearchResult {
                                    name: format!("{} (ShortID: {})", name, short_id),
                                    id: id.to_string(),
                                });
                            }
                        }
                    }
                    "MediaID" => {
                        // 搜索 MediaID 标签
                        for node in doc.descendants().filter(|n| n.has_tag_name("MediaID")) {
                            let media_id = node.attribute("ID").unwrap_or("");
                            if media_id.to_lowercase().contains(&query) {
                                // 获取父节点信息
                                if let Some(parent) = node.parent_element() {
                                    if let Some(grandparent) = parent.parent_element() {
                                        let name = grandparent.attribute("Name").unwrap_or("未命名");
                                        let id = grandparent.attribute("ID").unwrap_or(media_id);
                                        file_results.push(SearchResult {
                                            name: format!("{} (MediaID: {})", name, media_id),
                                            id: id.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            file_results
        })
        .collect();
    
    Ok(results)
}

/// 在 Bank 目录中搜索 ID
/// 
/// # 参数
/// * `directory` - Bank 目录路径（包含 SoundbanksInfo.xml 或 .json）
/// * `id_string` - 要搜索的 ID 字符串
/// * `id_types` - 要搜索的 ID 类型数组，可选值: ["GUID", "ShortID", "MediaID"]
/// 
/// # 返回
/// * `Ok(Vec<SearchResult>)` - 搜索结果列表
/// * `Err(String)` - 搜索失败，返回错误信息
#[tauri::command]
fn search_bank_directory(
    _directory: String,
    id_string: String,
    _id_types: Vec<String>,
) -> Result<Vec<SearchResult>, String> {
    // TODO: 实现 Bank 目录搜索逻辑
    // 1. 读取 SoundbanksInfo.xml 或 SoundbanksInfo.json
    // 2. 解析文件内容
    // 3. 根据 id_string 和 id_types 搜索匹配的对象
    // 4. 返回搜索结果
    
    // 示例返回（待实现）
    Ok(vec![
        SearchResult {
            name: format!("Bank 对象 - {}", id_string),
            id: id_string.clone(),
        }
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            validate_wwise_directory, 
            validate_bank_directory,
            search_wwise_project,
            search_bank_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
