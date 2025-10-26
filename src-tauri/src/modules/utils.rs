/// 验证是否为有效的GUID格式
/// GUID格式: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX} 或 XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX
pub fn is_valid_guid(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    // 移除可能的花括号
    let cleaned = s.trim_start_matches('{').trim_end_matches('}');
    
    // 检查格式: XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX
    if cleaned.len() != 36 {
        return false;
    }
    
    let parts: Vec<&str> = cleaned.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    
    // 检查每个部分的长度和字符
    if parts[0].len() != 8 || parts[1].len() != 4 || parts[2].len() != 4 || 
       parts[3].len() != 4 || parts[4].len() != 12 {
        return false;
    }
    
    // 检查是否所有字符都是十六进制
    for part in parts {
        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }
    
    true
}