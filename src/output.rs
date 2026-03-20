//! Dual output formatting: table (human-friendly) and JSON (machine-parseable).

use tabled::settings::Style;
use tabled::{Table, Tabled};

/// Output format selector, usable as a clap global flag.
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

/// Print any serializable value as pretty JSON.
pub fn print_json(data: &(impl serde::Serialize + ?Sized)) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

/// Print a vector of Tabled rows as a pretty table.
pub fn print_table<T: Tabled>(rows: &[T]) {
    if rows.is_empty() {
        println!("(no results)");
        return;
    }
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

/// Print a key-value detail table (2-column).
pub fn print_detail(rows: Vec<(&str, String)>) {
    if rows.is_empty() {
        println!("(no data)");
        return;
    }
    let data: Vec<[String; 2]> = rows
        .into_iter()
        .map(|(k, v)| [k.to_string(), v])
        .collect();
    let table = Table::from_iter(data).with(Style::rounded()).to_string();
    println!("{table}");
}

/// Format output based on OutputFormat: either JSON or call the table formatter.
pub fn render<T: serde::Serialize>(
    format: OutputFormat,
    data: &T,
    table_fn: impl FnOnce(&T),
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => print_json(data),
        OutputFormat::Table => {
            table_fn(data);
            Ok(())
        }
    }
}
