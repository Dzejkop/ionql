use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Mapping {
    pub section_mappings: HashMap<String, SectionMapping>,
}

impl Mapping {
    pub fn get(&self, section_name: impl ToString) -> Option<&SectionMapping> {
        self.section_mappings.get(&section_name.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct SectionMapping {
    pub row_name_to_idx: HashMap<String, usize>,
}

impl SectionMapping {
    pub fn get(&self, name: impl ToString) -> Option<usize> {
        self.row_name_to_idx.get(&name.to_string()).copied()
    }
}
