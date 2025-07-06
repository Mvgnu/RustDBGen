pub fn generate_pagination() -> String {
    "#[derive(Debug, Clone, Copy, serde::Deserialize)]\n".to_string() + 
    "pub struct Pagination { pub limit: i64, pub offset: i64 }\n\n"
} 