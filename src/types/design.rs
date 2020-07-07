use super::*;
use serde::{Serialize, Deserialize};
use find::{SortSpec};
use document::{DocumentId};

/// Design document created abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct DesignCreated {
    pub result: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub error: Option<String>,
    pub reason: Option<String>
}