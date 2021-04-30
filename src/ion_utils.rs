use anyhow::anyhow;
use ion::{Ion, Section, Value};

pub mod summary;

pub fn extract_fields_from_section<'a>(
    field_names: &[&str],
    section: &str,
    ion: &'a Ion,
) -> anyhow::Result<Vec<Vec<&'a Value>>> {
    let section = ion
        .get(section)
        .ok_or_else(|| anyhow!("Missing section {}", section))?;

    let header = section_header(&section)
        .ok_or_else(|| anyhow!("Missing section header"))?;

    let idxs_of_field_in_row: Vec<usize> =
        extract_field_idxs_from_rows(field_names, header)?;

    let items = section
        .rows_without_header()
        .iter()
        .map(|row| {
            idxs_of_field_in_row
                .iter()
                .filter_map(|idx| row.get(*idx))
                .collect()
        })
        .collect();

    Ok(items)
}

fn extract_field_idxs_from_rows(
    field_names: &[&str],
    header: &[Value],
) -> Result<Vec<usize>, anyhow::Error> {
    field_names
        .iter()
        .map(|field_name| {
            header
                .iter()
                .position(|header_field| {
                    &header_field.to_string() == field_name
                })
                .ok_or_else(|| {
                    anyhow!("Cannot find {} in {:?}", field_name, header)
                })
        })
        .collect::<Result<_, _>>()
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
