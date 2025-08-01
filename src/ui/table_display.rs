use anyhow::Result;
use csv::Writer;
use std::fs::File;
use std::io::Write;

use crate::database::QueryResult;

pub fn display_table(result: &QueryResult, max_rows: Option<usize>) {
    if result.is_empty() {
        println!("Query returned no results.");
        return;
    }

    let display_rows = if let Some(max) = max_rows {
        std::cmp::min(result.rows.len(), max)
    } else {
        result.rows.len()
    };

    // Create a simple table using format strings
    if !result.columns.is_empty() {
        // Calculate column widths
        let mut col_widths: Vec<usize> = result.columns
            .iter()
            .map(|col| col.len())
            .collect();

        for row in result.rows.iter().take(display_rows) {
            for (i, cell) in row.iter().enumerate() {
                if let Some(width) = col_widths.get_mut(i) {
                    *width = (*width).max(cell.len());
                }
            }
        }

        // Print header
        print!("┌");
        for (i, width) in col_widths.iter().enumerate() {
            print!("{}", "─".repeat(width + 2));
            if i < col_widths.len() - 1 {
                print!("┬");
            }
        }
        println!("┐");

        print!("│");
        for (i, (col, width)) in result.columns.iter().zip(&col_widths).enumerate() {
            print!(" {:<width$} ", col, width = width);
            if i < result.columns.len() - 1 {
                print!("│");
            }
        }
        println!("│");

        print!("├");
        for (i, width) in col_widths.iter().enumerate() {
            print!("{}", "─".repeat(width + 2));
            if i < col_widths.len() - 1 {
                print!("┼");
            }
        }
        println!("┤");

        // Print rows
        for row in result.rows.iter().take(display_rows) {
            print!("│");
            for (i, (cell, width)) in row.iter().zip(&col_widths).enumerate() {
                print!(" {:<width$} ", cell, width = width);
                if i < row.len() - 1 {
                    print!("│");
                }
            }
            println!("│");
        }

        print!("└");
        for (i, width) in col_widths.iter().enumerate() {
            print!("{}", "─".repeat(width + 2));
            if i < col_widths.len() - 1 {
                print!("┴");
            }
        }
        println!("┘");
    }

    if let Some(max) = max_rows {
        if result.rows.len() > max {
            println!("\n... and {} more rows (showing first {})", 
                result.rows.len() - max, max);
        }
    }

    println!("\nRows returned: {}", result.row_count);
}

pub fn export_to_csv(result: &QueryResult, file_path: &str) -> Result<()> {
    let file = File::create(file_path)?;
    let mut writer = Writer::from_writer(file);

    // Write headers
    writer.write_record(&result.columns)?;

    // Write data rows
    for row in &result.rows {
        writer.write_record(row)?;
    }

    writer.flush()?;
    println!("Results exported to: {}", file_path);
    Ok(())
}

pub fn export_to_json(result: &QueryResult, file_path: &str) -> Result<()> {
    let mut json_rows = Vec::new();
    
    for row in &result.rows {
        let mut json_row = serde_json::Map::new();
        for (i, column) in result.columns.iter().enumerate() {
            let value = row.get(i).unwrap_or(&"NULL".to_string()).clone();
            json_row.insert(column.clone(), serde_json::Value::String(value));
        }
        json_rows.push(serde_json::Value::Object(json_row));
    }

    let json_output = serde_json::Value::Array(json_rows);
    let mut file = File::create(file_path)?;
    file.write_all(serde_json::to_string_pretty(&json_output)?.as_bytes())?;
    
    println!("Results exported to: {}", file_path);
    Ok(())
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}
