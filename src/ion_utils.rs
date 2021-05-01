use anyhow::anyhow;
use ion::{Ion, Section, Value};
use itertools::Itertools;

use self::value_at_position::ValueAtPosition;
use crate::row_mapping::Mapping;

pub mod value_at_position;

#[derive(Debug)]
pub struct ResultBuilder<'a, 'n> {
    field_names: &'n [&'n str],
    rows: Vec<RowBuilder<'a>>,
    values_from_dicts: Vec<Option<&'a Value>>,
}

impl<'a, 'n> ResultBuilder<'a, 'n> {
    pub fn new(field_names: &'n [&'n str]) -> Self {
        Self {
            field_names,
            rows: vec![],
            values_from_dicts: vec![None; field_names.len()],
        }
    }

    pub fn finish(mut self) -> anyhow::Result<Vec<Vec<&'a Value>>> {
        let mut rows = vec![];

        for (idx, value) in self.values_from_dicts.into_iter().enumerate() {
            if let Some(value) = value {
                for builder in &mut self.rows {
                    builder.put(idx, value);
                }
            }
        }

        for builder in self.rows {
            rows.push(builder.finish()?);
        }

        Ok(rows)
    }
}

#[derive(Debug)]
pub struct RowBuilder<'a> {
    values: Vec<Option<&'a Value>>,
}

impl<'a> RowBuilder<'a> {
    pub fn new(len: usize) -> Self {
        Self {
            values: vec![None; len],
        }
    }

    pub fn put(&mut self, pos: usize, value: &'a Value) {
        *self.values.get_mut(pos).unwrap() = Some(value);
    }

    pub fn finish(self) -> anyhow::Result<Vec<&'a Value>> {
        if self.values.iter().any(|value| value.is_none()) {
            return Err(anyhow!("Failed to fill row"));
        }

        Ok(self.values.into_iter().flatten().collect())
    }
}

pub fn extract_fields_from_sections<'a>(
    field_names: &[&str],
    sections: &[&str],
    ion: &'a Ion,
    mapping: &Mapping,
) -> anyhow::Result<Vec<Vec<&'a Value>>> {
    let mut result_builder = ResultBuilder::new(field_names);

    for section_name in sections {
        let section = ion.get(section_name).ok_or_else(|| {
            anyhow!("Cannot find section {} in ion", section_name)
        })?;

        if section.rows.is_empty() {
            result_builder.extract_fields_from_dict_section(section)?;
        } else {
            result_builder.extract_fields_from_rows_section(
                section,
                section_name,
                mapping,
            )?;
        }
    }

    result_builder.finish()
}

impl<'a, 'n> ResultBuilder<'a, 'n> {
    fn extract_fields_from_dict_section(
        &mut self,
        section: &'a Section,
    ) -> anyhow::Result<()> {
        let dict_items = self
            .field_names
            .iter()
            .enumerate()
            .filter_map(|(idx, field_name)| {
                let value = section.dictionary.get(&field_name.to_string())?;

                Some(ValueAtPosition::new(idx, value))
            })
            .collect_vec();

        for value in dict_items {
            let pos = value.pos();
            *self.values_from_dicts.get_mut(pos).unwrap() =
                Some(value.take_value());
        }

        Ok(())
    }

    fn extract_fields_from_rows_section(
        &mut self,
        section: &'a Section,
        section_name: &str,
        mapping: &Mapping,
    ) -> anyhow::Result<()> {
        let idxs_of_fields_in_row =
            if let Some(mapping) = mapping.get(section_name) {
                self.field_names
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, field_name)| {
                        ValueAtPosition::new(idx, mapping.get(field_name))
                            .transpose()
                    })
                    .collect()
            } else {
                let header = section_header(&section)
                    .ok_or_else(|| anyhow!("Missing section header"))?;

                let idxs_of_fields_in_row: Vec<ValueAtPosition<usize>> =
                    extract_field_idxs_from_rows(self.field_names, header);

                idxs_of_fields_in_row
            };

        for (idx, row) in section.rows_without_header().iter().enumerate() {
            if self.rows.get(idx).is_none() {
                self.rows
                    .insert(idx, RowBuilder::new(self.field_names.len()));
            }
            let row_builder = self.rows.get_mut(idx).unwrap();

            for field_idx in &idxs_of_fields_in_row {
                if let Some(row_value) = row.get(*field_idx.value()) {
                    row_builder.put(field_idx.pos(), row_value);
                }
            }
        }

        Ok(())
    }
}

fn extract_field_idxs_from_rows(
    field_names: &[&str],
    header: &[Value],
) -> Vec<ValueAtPosition<usize>> {
    field_names
        .iter()
        .enumerate()
        .filter_map(|(idx, field_name)| {
            let idx_in_row = header.iter().position(|header_field| {
                &header_field.to_string() == field_name
            })?;

            Some(ValueAtPosition::new(idx, idx_in_row))
        })
        .collect()
}

fn section_header(section: &Section) -> Option<&[Value]> {
    if !section.rows.len() < 2 {
        return None;
    }

    if !is_row_header_separator(&section.rows[1]) {
        return None;
    }

    Some(&section.rows[0])
}

fn is_row_header_separator(row: &[Value]) -> bool {
    if !row.iter().all(|value| value.is_string()) {
        return false;
    }

    row.iter()
        .filter_map(|value| value.as_str())
        .all(|s| s.chars().all(|c| c == '-'))
}
