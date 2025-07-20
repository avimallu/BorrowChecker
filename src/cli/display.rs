use crate::core::receipt::{Receipt, SplittingError};
use comfy_table::{Cell, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};

impl Receipt {
    fn create_table(&self) -> Result<Table, SplittingError> {
        let mut header = self.shared_by.clone();
        header.insert(0, "Item".into());
        header.push("Total".into());

        let (item_names, item_splits) = self.calculate_splits()?;

        let rows: Vec<Vec<String>> = item_names
            .into_iter()
            .zip(item_splits.into_iter())
            .map(|(item_name, splits)| {
                let mut splits_as_str: Vec<String> = splits.iter().map(|x| x.to_string()).collect();
                splits_as_str.insert(0, item_name.into());
                splits_as_str
            })
            .collect();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(header);

        for (idx, column) in table.column_iter_mut().enumerate() {
            if idx != 0 {
                column.set_cell_alignment(comfy_table::CellAlignment::Right);
            }
        }

        for row in rows.iter() {
            if row[0] == "<total>" || row[0] == "<leftover>" {
                let fg_col = if row[0] == "<total>" {
                    comfy_table::Color::Green
                } else {
                    comfy_table::Color::DarkGrey
                };

                let row: Vec<Cell> = row.iter().map(|x| Cell::new(x).fg(fg_col)).collect();
                table.add_row(row);
            } else {
                table.add_row(row);
            }
        }

        Ok(table)
    }

    pub fn display_splits(&self) -> Result<(), SplittingError> {
        let table = self.create_table()?;
        print!("\n{table}\n");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::receipt::Receipt;
    use crate::utils;
    use rust_decimal::prelude::*;

    #[test]
    fn test_create_table() {
        let mut receipt = Receipt::new(
            Decimal::from_str("300").unwrap(),
            vec!["Alice", "Bob", "Marshall"],
        )
        .unwrap();
        receipt
            .add_item_split_by_ratio(
                dec![200],
                "Food".into(),
                utils::strs_to_strings(vec!["Alice", "Marshall", "Bob"]),
                None,
            )
            .unwrap();
        receipt
            .add_item_split_by_ratio(
                dec![50],
                "Drinks".into(),
                utils::strs_to_strings(vec!["Alice", "Bob"]),
                None,
            )
            .unwrap();

        println!(
            "Receipt Item Status Before Calculating Splits:\n\n{:#?}\n\n",
            receipt.items
        );

        let mut table = receipt.create_table().unwrap();

        table.force_no_tty();

        let expected = "
╭────────────┬────────┬────────┬──────────┬───────╮
│ Item       ┆  Alice ┆    Bob ┆ Marshall ┆ Total │
╞════════════╪════════╪════════╪══════════╪═══════╡
│ Food       ┆  66.67 ┆  66.67 ┆    66.67 ┆   200 │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ Drinks     ┆     25 ┆     25 ┆        0 ┆    50 │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ <leftover> ┆  18.33 ┆  18.33 ┆    13.33 ┆    50 │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ <total>    ┆ 110.00 ┆ 110.00 ┆    80.00 ┆   300 │
╰────────────┴────────┴────────┴──────────┴───────╯";
        let actual = "\n".to_string() + &table.to_string();
        assert_eq!(expected, actual)
    }
}
