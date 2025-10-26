use serde::{Deserialize, Serialize};

/// 搜索结果结构
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub object_type: String,
    pub guid: String,
    pub short_id: String,
    pub media_id: String,
}