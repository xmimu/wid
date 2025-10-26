use std::path::Path;
use std::fs;
use glob::glob;
use rayon::prelude::*;
use roxmltree::Document;
use crate::modules::types::SearchResult;
use crate::modules::utils::is_valid_guid;

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
pub fn search_wwise_project(
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
            
            // 检查输入是否为纯数字
            let is_numeric = id_string.chars().all(|c| c.is_ascii_digit());
            
            // 根据 id_types 执行不同的搜索
            for id_type in &id_types {
                match id_type.as_str() {
                    "GUID" => {
                        // 搜索 GUID (ID 属性)
                        for node in doc.descendants().filter(|n| n.has_attribute("ID")) {
                            let id = node.attribute("ID").unwrap_or("");
                            if is_valid_guid(id) && id.to_lowercase().contains(&query) {
                                let name = node.attribute("Name").unwrap_or("未命名");
                                let short_id = node.attribute("ShortID").unwrap_or("");
                                file_results.push(SearchResult {
                                    name: name.to_string(),
                                    object_type: node.tag_name().name().to_string(),
                                    guid: id.to_string(),
                                    short_id: short_id.to_string(),
                                    media_id: String::new(),
                                });
                            }
                        }
                    }
                    "ShortID" => {
                        // 搜索 ShortID 属性 - 只有纯数字才查询
                        if !is_numeric {
                            continue; // 跳过非数字输入
                        }
                        for node in doc.descendants().filter(|n| n.has_attribute("ShortID")) {
                            let short_id = node.attribute("ShortID").unwrap_or("");
                            if short_id.to_lowercase().contains(&query) {
                                let name = node.attribute("Name").unwrap_or("未命名");
                                let id = node.attribute("ID").unwrap_or("");
                                file_results.push(SearchResult {
                                    name: name.to_string(),
                                    object_type: node.tag_name().name().to_string(),
                                    guid: if is_valid_guid(id) { id.to_string() } else { String::new() },
                                    short_id: short_id.to_string(),
                                    media_id: String::new(),
                                });
                            }
                        }
                    }
                    "MediaID" => {
                        // 搜索 MediaID 标签 - 只有纯数字才查询
                        if !is_numeric {
                            continue; // 跳过非数字输入
                        }
                        for node in doc.descendants().filter(|n| n.has_tag_name("MediaID")) {
                            let media_id = node.attribute("ID").unwrap_or("");
                            if media_id.to_lowercase().contains(&query) {
                                // 获取父节点信息
                                if let Some(parent) = node.parent_element() {
                                    if let Some(grandparent) = parent.parent_element() {
                                        let name = grandparent.attribute("Name").unwrap_or("未命名");
                                        let id = grandparent.attribute("ID").unwrap_or("");
                                        let short_id = grandparent.attribute("ShortID").unwrap_or("");
                                        file_results.push(SearchResult {
                                            name: name.to_string(),
                                            object_type: grandparent.tag_name().name().to_string(),
                                            guid: if is_valid_guid(id) { id.to_string() } else { String::new() },
                                            short_id: short_id.to_string(),
                                            media_id: media_id.to_string(),
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