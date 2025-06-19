use std::path::PathBuf;
use polars::prelude::*;

pub fn open_csv_lazy(file: &PathBuf, field: &Option<String>) -> Result<LazyFrame, String> {
    LazyCsvReader::new(file)
        .with_infer_schema_length(Some(100))
        .with_has_header(true)
        .with_chunk_size(10000)
        .finish()
        .map_err(|e| format!("Failed to parse CSV file: {}", e))
        .map(|lf| match field {
            Some(field) => lf.select([col(field)]),
            None => lf
        }.with_new_streaming(true))
}

pub fn load_file(file: &PathBuf, field: &Option<String>) -> DataFrame {
    let mut lf = open_csv_lazy(file, field).unwrap();
    let schema = lf.collect_schema().unwrap();

    // Collect all the columns that have an analysis
    let mut names = Vec::new();
    for (name, dtype) in schema.iter() {
        if dtype == &DataType::String {
            names.push(name.to_string());
        }
    }

    // Build an expression for the columns
    let expr = names
        .iter()
        .map(|name| col(name.to_string()))
        .collect::<Vec<_>>();

    lf.select(expr).collect().unwrap()
}