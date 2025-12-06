//! Model interface types extracted from WIT definitions.

use std::fmt;

/// Complete interface definition for a model.
#[derive(Debug, Clone)]
pub struct ModelInterface {
    /// Model name (e.g., "drag")
    pub name: String,
    /// WIT package (e.g., "myorg:physics")
    pub package: String,
    /// Constructor parameters record
    pub params: RecordDef,
    /// Step function input record
    pub inputs: RecordDef,
    /// Step function output record
    pub outputs: RecordDef,
}

/// Definition of a WIT record type.
#[derive(Debug, Clone)]
pub struct RecordDef {
    /// Record name in PascalCase
    pub name: String,
    /// Fields in the record
    pub fields: Vec<FieldDef>,
}

/// Definition of a single field in a record.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// Field name in snake_case (for generated code)
    pub name: String,
    /// Original WIT name (kebab-case)
    pub wit_name: String,
    /// Field type
    pub ty: WitType,
    /// Documentation comment, if any
    pub docs: Option<String>,
}

/// Supported WIT primitive and compound types.
#[derive(Debug, Clone, PartialEq)]
pub enum WitType {
    Float64,
    Float32,
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    String,
    Bool,
    Char,
    List(Box<WitType>),
    Option(Box<WitType>),
    Tuple(Vec<WitType>),
    // Future: Result, Variant, Flags, etc.
}

impl WitType {
    /// Convert WIT type to Python type annotation.
    pub fn to_python(&self) -> String {
        match self {
            WitType::Float64 | WitType::Float32 => "float".into(),
            WitType::U8 | WitType::U16 | WitType::U32 | WitType::U64 => "int".into(),
            WitType::S8 | WitType::S16 | WitType::S32 | WitType::S64 => "int".into(),
            WitType::String => "str".into(),
            WitType::Bool => "bool".into(),
            WitType::Char => "str".into(),
            WitType::List(inner) => format!("list[{}]", inner.to_python()),
            WitType::Option(inner) => format!("{} | None", inner.to_python()),
            WitType::Tuple(items) => {
                let inner: Vec<_> = items.iter().map(|t| t.to_python()).collect();
                format!("tuple[{}]", inner.join(", "))
            }
        }
    }

    /// Convert WIT type to Rust type.
    pub fn to_rust(&self) -> String {
        match self {
            WitType::Float64 => "f64".into(),
            WitType::Float32 => "f32".into(),
            WitType::U8 => "u8".into(),
            WitType::U16 => "u16".into(),
            WitType::U32 => "u32".into(),
            WitType::U64 => "u64".into(),
            WitType::S8 => "i8".into(),
            WitType::S16 => "i16".into(),
            WitType::S32 => "i32".into(),
            WitType::S64 => "i64".into(),
            WitType::String => "String".into(),
            WitType::Bool => "bool".into(),
            WitType::Char => "char".into(),
            WitType::List(inner) => format!("Vec<{}>", inner.to_rust()),
            WitType::Option(inner) => format!("Option<{}>", inner.to_rust()),
            WitType::Tuple(items) => {
                let inner: Vec<_> = items.iter().map(|t| t.to_rust()).collect();
                format!("({})", inner.join(", "))
            }
        }
    }
}

impl fmt::Display for WitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitType::Float64 => write!(f, "float64"),
            WitType::Float32 => write!(f, "float32"),
            WitType::U8 => write!(f, "u8"),
            WitType::U16 => write!(f, "u16"),
            WitType::U32 => write!(f, "u32"),
            WitType::U64 => write!(f, "u64"),
            WitType::S8 => write!(f, "s8"),
            WitType::S16 => write!(f, "s16"),
            WitType::S32 => write!(f, "s32"),
            WitType::S64 => write!(f, "s64"),
            WitType::String => write!(f, "string"),
            WitType::Bool => write!(f, "bool"),
            WitType::Char => write!(f, "char"),
            WitType::List(inner) => write!(f, "list<{}>", inner),
            WitType::Option(inner) => write!(f, "option<{}>", inner),
            WitType::Tuple(items) => {
                let inner: Vec<_> = items.iter().map(|t| t.to_string()).collect();
                write!(f, "tuple<{}>", inner.join(", "))
            }
        }
    }
}
