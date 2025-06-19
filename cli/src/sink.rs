use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use polars::{frame::DataFrame, prelude::AnyValue};

pub fn print_dataframe(df: &DataFrame) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset("     ──            ");
    
    // Add headers
    let headers: Vec<Cell> = df
        .get_column_names()
        .iter()
        .map(|s| Cell::new(s)
            .add_attribute(Attribute::Bold))
        .collect();
    table.set_header(headers);

    // Add data rows
    for i in 0..df.height() {
        let row = df.get_row(i).unwrap();
        let row_data: Vec<Cell> = row.0
            .iter()
            .map(|val| format_cell(val))
            .collect();

        table.add_row(row_data);
    }

    println!("\n");
    println!("{}", table);
    println!("\n");
}

fn format_cell(val: &polars::prelude::AnyValue) -> Cell {
    match val {
        AnyValue::Int64(_) | AnyValue::Int32(_) | AnyValue::Int16(_) | AnyValue::Int8(_) |
        AnyValue::UInt64(_) | AnyValue::UInt32(_) | AnyValue::UInt16(_) | AnyValue::UInt8(_) |
        AnyValue::Float64(_) | AnyValue::Float32(_) => {
            Cell::new(val.to_string())
                .set_alignment(CellAlignment::Right)
        },
        AnyValue::String(s) => {
            Cell::new(s.to_string())
        }
        _ => {
            Cell::new(val.to_string())
        }
    }
}