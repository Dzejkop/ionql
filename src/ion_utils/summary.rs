use std::collections::HashMap;

pub struct Summary {
    sections: Vec<String>,
    field_to_section_row_idx: HashMap<String, usize>,
}
