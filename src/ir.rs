use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    #[default]
    Postgres,
    Mysql,
    Sqlite,
}
use toml;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaIR {
    pub schema_version: String,
    pub meta: Meta,
    #[serde(default)]
    pub enums: HashMap<String, EnumDef>,
    #[serde(default)]
    pub models: HashMap<String, ModelDef>,
    #[serde(default)]
    pub routes: HashMap<String, RouteDef>,
    #[serde(default)]
    pub plugins: HashMap<String, PluginDef>,
    #[serde(default)]
    pub macros: HashMap<String, MacroDef>,
    #[serde(default)]
    pub seeds: HashMap<String, SeedDef>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Meta {
    pub rust_case_style: String,
    pub db_case_style: String,
    pub default_derives: Vec<String>,
    #[serde(default)]
    pub observability_provider: Option<String>,
    #[serde(default)]
    pub db_backend: DatabaseBackend,
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    #[default]
    None,
    Jwt,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
    #[serde(default)]
    pub provider: AuthProvider,
    #[serde(default = "default_anonymous_role")]
    pub anonymous_role: String,
    /// JWT claim key that stores the user's role
    #[serde(default = "default_role_claim")]
    pub role_claim: String,
    /// Special role name that grants public access
    #[serde(default = "default_public_role")]
    pub public_role: String,
}

fn default_anonymous_role() -> String {
    "guest".to_string()
}

fn default_public_role() -> String {
    "public".to_string()
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            provider: AuthProvider::None,
            anonymous_role: default_anonymous_role(),
            role_claim: default_role_claim(),
            public_role: default_public_role(),
        }
    }
}

fn default_role_claim() -> String {
    "role".to_string()
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: Vec::new(),
            observability_provider: None,
            db_backend: DatabaseBackend::Postgres,
            auth: AuthConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnumDef {
    pub variants: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TypeAlias {
    pub rust_type: String,
    pub db_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelDef {
    #[serde(default)]
    pub includes: Vec<String>,
    pub fields: HashMap<String, FieldDef>,
    #[serde(default)]
    pub indexes: HashMap<String, IndexDef>,
    #[serde(default)]
    pub relations: HashMap<String, RelationDef>,
    #[serde(default)]
    pub unique_constraints: HashMap<String, UniqueConstraintDef>,
    #[serde(default)]
    pub check_constraints: HashMap<String, CheckConstraintDef>,
    #[serde(default)]
    pub exclusion_constraints: HashMap<String, ExclusionConstraintDef>,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub options: ModelOptions,
    /// Indicates which model owns this resource (for authorization purposes)
    #[serde(default)]
    pub owned_by: Option<String>,
}

impl Default for ModelDef {
    fn default() -> Self {
        ModelDef {
            includes: Vec::new(),
            fields: HashMap::new(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Permissions::default(),
            options: ModelOptions::default(),
            owned_by: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelOptions {
    #[serde(default)]
    pub timestamps: bool,
    #[serde(default)]
    pub soft_delete: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MacroDef {
    #[serde(default)]
    pub fields: HashMap<String, FieldDef>,
    #[serde(default)]
    pub options: ModelOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct StorageOptions {
    pub backend: String,
    #[serde(default)]
    pub max_size: Option<String>,
    #[serde(default)]
    pub allowed_types: Vec<String>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldDef {
    #[serde(rename = "type")]
    pub rust_type: String,
    #[serde(default)]
    pub db_type: Option<String>,
    pub default: Option<String>,
    #[serde(default)]
    pub nullable: bool,
    /// When set, indicates this field was renamed from the given column name
    #[serde(default)]
    pub rename_from: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional custom Zod validation expression
    #[serde(default)]
    pub zod: Option<String>,
    #[serde(default)]
    pub storage: Option<StorageOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexDef {
    pub fields: Vec<String>,
    #[serde(default)]
    pub unique: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct FieldRef {
    pub model: String,
    pub field: String,
}

impl<'de> serde::Deserialize<'de> for FieldRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Helper {
            Struct { model: String, field: String },
            String(String),
        }
        match Helper::deserialize(deserializer)? {
            Helper::Struct { model, field } => Ok(FieldRef { model, field }),
            Helper::String(s) => {
                let parts: Vec<&str> = s.split('.').collect();
                if parts.len() == 2 {
                    Ok(FieldRef { model: parts[0].to_string(), field: parts[1].to_string() })
                } else {
                    Err(serde::de::Error::custom("expected `Model.field`"))
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationDef {
    pub on: String,
    pub references: FieldRef,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UniqueConstraintDef {
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckConstraintDef {
    pub expression: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExclusionConstraintDef {
    /// Everything after `EXCLUDE` in the constraint definition
    pub definition: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Permissions {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub update: Vec<String>,
    #[serde(default)]
    pub delete: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RouteDef {
    pub methods: Vec<String>,
    pub path: String,
    #[serde(default)]
    pub auth_required: bool,
    #[serde(default)]
    pub permissions: Permissions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginDef {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables to set when running the plugin
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Working directory when running the plugin
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SeedDef {
    #[serde(default)]
    pub rows: Vec<HashMap<String, toml::Value>>,
}
