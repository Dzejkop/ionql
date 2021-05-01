use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context};
use config::Config;
use ion::{Ion, Value};
use itertools::Itertools;
use sqlparser::ast::{
    Expr, Query, Select, SelectItem, SetExpr, Statement, TableFactor,
};
use sqlparser::dialect::GenericDialect;

pub mod config;
mod ion_utils;
mod row_mapping;

pub fn parse_query(s: &str) -> anyhow::Result<Box<Select>> {
    let dialect = GenericDialect {};

    let query = sqlparser::parser::Parser::parse_sql(&dialect, s)
        .context("Failed to parse query")?;

    let query = query
        .into_iter()
        .exactly_one()
        .map_err(|_| anyhow!("Expected a single query"))?;

    let query = match query {
        Statement::Query(query) => query,
        _ => return Err(anyhow!("Invalid SQL, expected a query")),
    };

    let Query {
        fetch,
        limit,
        with,
        offset,
        order_by,
        ..
    } = query.as_ref();

    unsupported_field(fetch.as_ref(), "FETCH")?;
    unsupported_field(limit.as_ref(), "LIMIT")?;
    unsupported_field(with.as_ref(), "WITH")?;
    unsupported_field(offset.as_ref(), "OFFSET")?;
    if !order_by.is_empty() {
        return Err(anyhow!("ORDER BY is not supported"));
    }

    let body = match query.body {
        SetExpr::Select(select) => select,
        _ => return Err(anyhow!("Only selects are supported")),
    };

    Ok(body)
}

pub fn query_file(
    path: impl AsRef<Path>,
    query: &str,
    config: &Config,
) -> anyhow::Result<Vec<Vec<Value>>> {
    let path = path.as_ref();

    let content = fs::read_to_string(path).with_context(|| {
        format!("Failed to read content from {}", path.display())
    })?;

    query_content(&content, query, config)
}

pub fn query_content(
    content: &str,
    query: &str,
    config: &Config,
) -> anyhow::Result<Vec<Vec<Value>>> {
    let ion: Ion = content.parse().context("Failed to parse content as ion")?;

    let results = query_ion(&ion, query, config)?;

    Ok(results
        .into_iter()
        .map(|row| row.into_iter().cloned().collect())
        .collect())
}

pub fn query_ion<'a>(
    ion: &'a Ion,
    query: &str,
    config: &Config,
) -> anyhow::Result<Vec<Vec<&'a Value>>> {
    let query = parse_query(query)?;

    let section_names = query
        .from
        .iter()
        .map(|from| match &from.relation {
            TableFactor::Table { name, .. } => {
                Ok(name.0.iter().map(|x| x.value.as_str()).join("."))
            }
            _ => Err(anyhow!("Only concrete section names are allowed")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let section_names_str =
        section_names.iter().map(|s| s.as_str()).collect::<Vec<_>>();

    let proj = query
        .projection
        .iter()
        .map(|proj| match proj {
            SelectItem::UnnamedExpr(Expr::Identifier(identifier)) => {
                Ok(identifier.value.as_str())
            }
            _ => Err(anyhow!("Unsupported projection")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let result_values = ion_utils::extract_fields_from_sections(
        &proj,
        &section_names_str,
        &ion,
        &config.mappings,
    )?;

    Ok(result_values)
}

fn unsupported_field<T>(field: Option<T>, name: &str) -> anyhow::Result<()> {
    if field.is_some() {
        Err(anyhow!("{} is not supported", name))
    } else {
        Ok(())
    }
}
