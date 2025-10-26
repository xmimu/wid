use std::path::Path;
use std::fs;
use roxmltree::Document;
use serde_json::Value;
use crate::modules::types::SearchResult;
use crate::modules::utils::is_valid_guid;

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
pub fn search_bank_directory(
    directory: String,
    id_string: String,
    id_types: Vec<String>,
) -> Result<Vec<SearchResult>, String> {
    // 验证目录
    let dir_path = Path::new(&directory);
    if !dir_path.is_dir() {
        return Err("目录不存在".to_string());
    }
    
    // 检查输入是否为纯数字
    let is_numeric = id_string.chars().all(|c| c.is_ascii_digit());
    let query = id_string.to_lowercase();
    
    // 查找 SoundbanksInfo 文件
    let json_path = dir_path.join("SoundbanksInfo.json");
    let xml_path = dir_path.join("SoundbanksInfo.xml");
    
    let mut results = Vec::new();
    
    if json_path.exists() {
        // 解析 JSON 文件
        match parse_soundbanks_json(&json_path, &query, &id_types, is_numeric) {
            Ok(mut json_results) => results.append(&mut json_results),
            Err(e) => return Err(format!("解析 JSON 文件失败: {}", e)),
        }
    } else if xml_path.exists() {
        // 解析 XML 文件
        match parse_soundbanks_xml(&xml_path, &query, &id_types, is_numeric) {
            Ok(mut xml_results) => results.append(&mut xml_results),
            Err(e) => return Err(format!("解析 XML 文件失败: {}", e)),
        }
    } else {
        return Err("未找到 SoundbanksInfo.json 或 SoundbanksInfo.xml".to_string());
    }
    
    Ok(results)
}

/// 解析 SoundbanksInfo.xml 文件
fn parse_soundbanks_xml(
    file_path: &Path,
    query: &str,
    id_types: &[String],
    is_numeric: bool,
) -> Result<Vec<SearchResult>, String> {
    // 读取文件内容
    let contents = fs::read_to_string(file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 解析 XML
    let doc = Document::parse(&contents)
        .map_err(|e| format!("XML 解析失败: {}", e))?;
    
    let mut results = Vec::new();
    
    // 获取根节点 SoundBanksInfo
    if let Some(root) = doc.root_element().children().find(|n| n.has_tag_name("SoundBanks")) {
        // 遍历所有 SoundBank
        for soundbank in root.children().filter(|n| n.has_tag_name("SoundBank")) {
            search_in_xml_node(&soundbank, query, id_types, is_numeric, &mut results, "SoundBank");
        }
    }
    
    Ok(results)
}

/// 递归搜索 XML 节点中的匹配项
fn search_in_xml_node(
    node: &roxmltree::Node,
    query: &str,
    id_types: &[String],
    is_numeric: bool,
    results: &mut Vec<SearchResult>,
    node_type: &str,
) {
    // 提取节点属性信息
    let name = node.attribute("Name")
        .or_else(|| node.attribute("ShortName"))
        .or_else(|| {
            // 如果没有 Name 属性，尝试从子节点获取
            node.children().find(|n| n.has_tag_name("ShortName") || n.has_tag_name("ObjectPath"))
                .and_then(|n| n.text())
        })
        .unwrap_or("未命名");
    
    let guid = node.attribute("GUID").unwrap_or("");
    let id = node.attribute("Id").unwrap_or("");
    
    // 根据 id_types 进行匹配
    for id_type in id_types {
        match id_type.as_str() {
            "GUID" => {
                if !guid.is_empty() && is_valid_guid(guid) && guid.to_lowercase().contains(query) {
                    results.push(SearchResult {
                        name: name.to_string(),
                        object_type: node_type.to_string(),
                        guid: guid.to_string(),
                        short_id: id.to_string(),
                        media_id: String::new(),
                    });
                }
            }
            "ShortID" => {
                if is_numeric && !id.is_empty() && id.to_lowercase().contains(query) {
                    results.push(SearchResult {
                        name: name.to_string(),
                        object_type: node_type.to_string(),
                        guid: if is_valid_guid(guid) { guid.to_string() } else { String::new() },
                        short_id: id.to_string(),
                        media_id: String::new(),
                    });
                }
            }
            "MediaID" => {
                if is_numeric && !id.is_empty() && id.to_lowercase().contains(query) {
                    // 对于 Media 对象，使用 ShortName 作为 Name，ID 作为 MediaID
                    if node_type == "Media" {
                        let short_name = node.attribute("ShortName").unwrap_or(name);
                        results.push(SearchResult {
                            name: short_name.to_string(),
                            object_type: "Media".to_string(),
                            guid: String::new(), // Media对象的ID不是GUID格式，所以保持为空
                            short_id: String::new(),
                            media_id: id.to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    
    // 递归搜索子节点
    for child in node.children().filter(|n| n.is_element()) {
        let child_type = match child.tag_name().name() {
            "Event" => "Event",
            "File" => "Media",  // XML 中的 Media 对象标签名是 File
            "GameParameter" => "GameParameter",
            "StateGroup" => "StateGroup",
            "State" => "State",
            "SwitchGroup" => "SwitchGroup",
            "Switch" => "Switch",
            "Bus" => "Bus",
            "AcousticTexture" => "AcousticTexture",
            "Plugin" => "Plugin",
            "AudioDevices" => {
                // AudioDevices 容器，子节点是 Plugin
                for plugin in child.children().filter(|n| n.has_tag_name("Plugin")) {
                    search_in_xml_node(&plugin, query, id_types, is_numeric, results, "AudioDevice");
                }
                continue;
            }
            "Custom" => {
                // Custom 容器，子节点是 Plugin
                for plugin in child.children().filter(|n| n.has_tag_name("Plugin")) {
                    search_in_xml_node(&plugin, query, id_types, is_numeric, results, "CustomPlugin");
                }
                continue;
            }
            "GameParameters" => {
                // GameParameters 容器，子节点是 GameParameter
                for param in child.children().filter(|n| n.has_tag_name("GameParameter")) {
                    search_in_xml_node(&param, query, id_types, is_numeric, results, "GameParameter");
                }
                continue;
            }
            "StateGroups" => {
                // StateGroups 容器，子节点是 StateGroup
                for group in child.children().filter(|n| n.has_tag_name("StateGroup")) {
                    search_in_xml_node(&group, query, id_types, is_numeric, results, "StateGroup");
                }
                continue;
            }
            "States" => {
                // States 容器，子节点是 State
                for state in child.children().filter(|n| n.has_tag_name("State")) {
                    search_in_xml_node(&state, query, id_types, is_numeric, results, "State");
                }
                continue;
            }
            "Busses" => {
                // Busses 容器，子节点是 Bus
                for bus in child.children().filter(|n| n.has_tag_name("Bus")) {
                    search_in_xml_node(&bus, query, id_types, is_numeric, results, "Bus");
                }
                continue;
            }
            "AcousticTextures" => {
                // AcousticTextures 容器，子节点是 AcousticTexture
                for texture in child.children().filter(|n| n.has_tag_name("AcousticTexture")) {
                    search_in_xml_node(&texture, query, id_types, is_numeric, results, "AcousticTexture");
                }
                continue;
            }
            "Events" => {
                // Events 容器，子节点是 Event
                for event in child.children().filter(|n| n.has_tag_name("Event")) {
                    search_in_xml_node(&event, query, id_types, is_numeric, results, "Event");
                }
                continue;
            }
            "Media" => {
                // Media 容器，子节点是 File
                for file in child.children().filter(|n| n.has_tag_name("File")) {
                    search_in_xml_node(&file, query, id_types, is_numeric, results, "Media");
                }
                continue;
            }
            "SwitchContainers" => {
                // SwitchContainers 容器，子节点是 SwitchContainer
                for container in child.children().filter(|n| n.has_tag_name("SwitchContainer")) {
                    search_in_xml_node(&container, query, id_types, is_numeric, results, "SwitchContainer");
                }
                continue;
            }
            tag_name => tag_name,
        };
        
        search_in_xml_node(&child, query, id_types, is_numeric, results, child_type);
    }
}

/// 解析 SoundbanksInfo.json 文件
fn parse_soundbanks_json(
    file_path: &Path,
    query: &str,
    id_types: &[String],
    is_numeric: bool,
) -> Result<Vec<SearchResult>, String> {
    // 读取文件内容
    let contents = fs::read_to_string(file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 解析 JSON
    let json: Value = serde_json::from_str(&contents)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    
    let mut results = Vec::new();
    
    // 获取 SoundBanksInfo 根节点
    if let Some(soundbanks_info) = json.get("SoundBanksInfo") {
        if let Some(soundbanks) = soundbanks_info.get("SoundBanks").and_then(|v| v.as_array()) {
            for soundbank in soundbanks {
                // 搜索每个 SoundBank
                search_in_value(soundbank, query, id_types, is_numeric, &mut results, "SoundBank");
            }
        }
    }
    
    Ok(results)
}

/// 递归搜索 JSON 值中的匹配项
fn search_in_value(
    value: &Value,
    query: &str,
    id_types: &[String],
    is_numeric: bool,
    results: &mut Vec<SearchResult>,
    parent_type: &str,
) {
    match value {
        Value::Object(obj) => {
            // 提取基础信息
            let name = obj.get("Name")
                .or_else(|| obj.get("ShortName"))
                .and_then(|v| v.as_str())
                .unwrap_or("未命名");
            
            let guid = obj.get("GUID")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let id = if let Some(id_str) = obj.get("Id").and_then(|v| v.as_str()) {
                id_str.to_string()
            } else if let Some(id_num) = obj.get("Id").and_then(|v| v.as_u64()) {
                id_num.to_string()
            } else {
                String::new()
            };
            
            // 根据 id_types 进行匹配
            for id_type in id_types {
                match id_type.as_str() {
                    "GUID" => {
                        if !guid.is_empty() && is_valid_guid(guid) && guid.to_lowercase().contains(query) {
                            results.push(SearchResult {
                                name: name.to_string(),
                                object_type: parent_type.to_string(),
                                guid: guid.to_string(),
                                short_id: id.clone(),
                                media_id: String::new(),
                            });
                        }
                    }
                    "ShortID" => {
                        if is_numeric && !id.is_empty() && id.to_lowercase().contains(query) {
                            results.push(SearchResult {
                                name: name.to_string(),
                                object_type: parent_type.to_string(),
                                guid: if is_valid_guid(guid) { guid.to_string() } else { String::new() },
                                short_id: id.clone(),
                                media_id: String::new(),
                            });
                        }
                    }
                    "MediaID" => {
                        if is_numeric && !id.is_empty() && id.to_lowercase().contains(query) {
                            // 对于 Media 对象，使用 ShortName 作为 Name，ID 作为 MediaID
                            if parent_type == "Media" {
                                results.push(SearchResult {
                                    name: name.to_string(),
                                    object_type: "Media".to_string(),
                                    guid: String::new(), // Media对象的ID不是GUID格式，所以保持为空
                                    short_id: String::new(),
                                    media_id: id.clone(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // 递归搜索子对象
            for (key, val) in obj {
                let child_type = match key.as_str() {
                    "Events" => "Event",
                    "Media" => "Media",
                    "GameParameters" => "GameParameter",
                    "StateGroups" => "StateGroup",
                    "States" => "State",
                    "SwitchGroups" => "SwitchGroup",
                    "Switches" => "Switch",
                    "Busses" => "Bus",
                    "AcousticTextures" => "AcousticTexture",
                    "Plugins" => "Plugin",
                    "Custom" => "CustomPlugin",
                    "AudioDevices" => "AudioDevice",
                    "SwitchContainers" => "SwitchContainer",
                    "SwitchValue" => "SwitchValue",
                    _ => key,
                };
                
                search_in_value(val, query, id_types, is_numeric, results, child_type);
            }
        }
        Value::Array(arr) => {
            // 递归搜索数组中的每个元素
            for item in arr {
                search_in_value(item, query, id_types, is_numeric, results, parent_type);
            }
        }
        _ => {
            // 基础类型，不需要处理
        }
    }
}