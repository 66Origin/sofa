use super::*;
use serde::{Serialize, Deserialize};
use find::{SortSpec};
use document::{DocumentId};

/// Index fields abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct IndexFields {
    pub fields: Vec<SortSpec>
}

impl IndexFields {
    pub fn new(fields: Vec<SortSpec>) -> IndexFields {
        IndexFields {
            fields
        }
    }
}

/// Index abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Index {
    pub ddoc: Option<DocumentId>,
    pub name: String,
    #[serde(rename = "type")]
    pub index_type: String,
    pub def: IndexFields
}

/// Index created abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct IndexCreated {
    pub result: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub error: Option<String>,
    pub reason: Option<String>
}

/// Database index list abstraction
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DatabaseIndexList {
    pub total_rows: u32,
    pub indexes: Vec<Index>
}
