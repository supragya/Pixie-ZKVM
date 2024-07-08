use plonky2::hash::hash_types::RichField;
use prettytable::Table;

pub fn debug_table<F: RichField, const COLS: usize>(
    table_name: &str,
    headings: [&str; COLS],
    values: &Vec<[F; COLS]>,
) {
    let mut table = Table::new();
    table.add_row(headings.into());
    for row in values {
        table.add_row(row.into());
    }
    println!("TRACE OUTPUT: {}\n", table_name);
    table.printstd();
}
