use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context};
use ion::{Ion, Value};
use itertools::Itertools;
use sqlparser::ast::{
    Expr, Query, SelectItem, SetExpr, Statement, TableFactor,
};
use sqlparser::dialect::GenericDialect;

mod ion_utils;

pub fn query_file(
    path: impl AsRef<Path>,
    query: &str,
) -> anyhow::Result<Vec<Vec<Value>>> {
    let path = path.as_ref();

    let content = fs::read_to_string(path).with_context(|| {
        format!("Failed to read content from {}", path.display())
    })?;

    query_content(&content, query)
}

pub fn query_content(
    content: &str,
    query: &str,
) -> anyhow::Result<Vec<Vec<Value>>> {
    let ion: Ion = content.parse().context("Failed to parse content as ion")?;

    let results = query_ion(&ion, query)?;

    Ok(results
        .into_iter()
        .map(|row| row.into_iter().map(|x| x.clone()).collect())
        .collect())
}

pub fn query_ion<'a>(
    ion: &'a Ion,
    query: &str,
) -> anyhow::Result<Vec<Vec<&'a Value>>> {
    let dialect = GenericDialect {};

    let query = sqlparser::parser::Parser::parse_sql(&dialect, query)
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
        body,
        fetch,
        limit,
        with,
        offset,
        order_by,
    } = query.as_ref();

    unsupported_field(fetch.as_ref(), "FETCH")?;
    unsupported_field(limit.as_ref(), "LIMIT")?;
    unsupported_field(with.as_ref(), "WITH")?;
    unsupported_field(offset.as_ref(), "OFFSET")?;
    if !order_by.is_empty() {
        return Err(anyhow!("ORDER BY is not supported"));
    }

    let body = match body {
        SetExpr::Select(select) => select,
        _ => return Err(anyhow!("Only selects are supported")),
    };

    let section_names = body
        .from
        .iter()
        .map(|from| match &from.relation {
            TableFactor::Table { name, .. } => {
                Ok(name.0.iter().map(|x| x.value.as_str()).join("."))
            }
            _ => Err(anyhow!("Only concrete section names are allowed")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let proj = body
        .projection
        .iter()
        .map(|proj| match proj {
            SelectItem::UnnamedExpr(Expr::Identifier(identifier)) => {
                Ok(identifier.value.as_str())
            }
            _ => Err(anyhow!("Unsupported projection")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let results =
        ion_utils::extract_fields_from_section(&proj, &section_names[0], &ion)?;

    Ok(results)
}

fn unsupported_field<T>(field: Option<T>, name: &str) -> anyhow::Result<()> {
    if field.is_some() {
        Err(anyhow!("{} is not supported", name))
    } else {
        Ok(())
    }
}
