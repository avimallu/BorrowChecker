use crate::cli::utils as parse_utils;
use crate::core::receipt::{Receipt, SplittingError};
use crate::utils;
use rust_decimal::Decimal;

// Contains any pattern based parsing of inputs for the package.

impl Receipt {
    pub fn parse_create_receipt(amount_shared_by: &str) -> Result<Receipt, SplittingError> {
        let (total, shared_by) = parse_utils::split_by_comma(
            amount_shared_by,
            "Input must have pattern 'Total,Person_1[,Person_2,...]', but you have not provided the starting comma.",
        )?;
        let total = total.parse()?;
        let shared_by: Vec<&str> = shared_by.split(",").collect();
        Receipt::new(total, shared_by)
    }

    fn align_to_shared_by(&mut self, abbrevs: &str) -> Result<Vec<String>, SplittingError> {
        let abbrevs: Vec<&str> = abbrevs.split(",").collect();

        utils::is_string_vec_unique(
            &abbrevs,
            SplittingError::InvalidAbbreviation(format!(
                "The abbreviation string: {} has duplicates.",
                abbrevs.join(",")
            )),
        )?;

        let mut matched_names: Vec<String> = Vec::new();

        // Case is important - Don vs. don can be considered different people.
        // Minimal disruption to user, less code to peruse.
        for abbrev in abbrevs {
            // If the abbreviation is already mapped to an existing name:
            if let Some(existing_name) = self.mapped_abbreviations.get(abbrev) {
                // If it doesn't map to another one, add this as a mapped name.
                if matched_names.contains(existing_name) {
                    return Err(SplittingError::DuplicatePeopleError(format!(
                        "{} maps to {}, which has already been specified once.",
                        abbrev, existing_name
                    )));
                } else {
                    matched_names.push(existing_name.clone());
                }
            } else {
                // If the abbreviation is not mapped, try to find a map.
                let mut found = false;

                for name in &self.shared_by {
                    if utils::is_abbrev_match_to_string(abbrev, name)
                        & !matched_names.contains(name)
                    {
                        self.mapped_abbreviations
                            .insert(abbrev.to_string(), name.clone());
                        found = true;
                        matched_names.push(name.clone());
                        break;
                    }
                }

                // Not finding a match is an error.
                if !found {
                    return Err(SplittingError::InvalidAbbreviation(format!(
                        "{} does not match to a provided person name.",
                        abbrev
                    )));
                }
            }
        }

        Ok(matched_names)
    }

    pub fn parse_add_named_item(
        &mut self,
        item_name: &str,
        item_pattern: &str,
    ) -> Result<(), SplittingError> {
        let (value, abbrevs) = parse_utils::split_by_comma(
            item_pattern,
            &format!(
                "The first argument must have pattern 'Value,Person_1[,Person_2,...]', but you have {}",
                item_pattern
            ),
        )?;
        let value: Decimal = value.parse()?;
        let shared_by = self.align_to_shared_by(&abbrevs)?;
        // Todo: Add parsing of ratios specified in item names
        self.add_item_split_by_ratio(value, item_name.to_string(), shared_by, None)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::core::receipt::{Receipt, SplittingError};
    use rust_decimal::prelude::*;

    #[test]
    fn test_no_people_to_share_with() {
        let receipt = Receipt::parse_create_receipt("300,");
        assert!(matches!(
            receipt,
            Err(SplittingError::NotEnoughPeopleError(_))
        ))
    }

    #[test]
    fn test_only_one_person_to_share_with() {
        let receipt = Receipt::parse_create_receipt("300,Alice");
        assert!(matches!(
            receipt,
            Err(SplittingError::NotEnoughPeopleError(_))
        ))
    }

    #[test]
    fn test_non_decimal_total() {
        let receipt = Receipt::parse_create_receipt("wowza,Alice");
        assert!(matches!(
            receipt,
            Err(SplittingError::DecimalParsingError(_))
        ))
    }

    #[test]
    fn test_two_people() {
        let receipt = Receipt::parse_create_receipt("300,Alice,Sam").unwrap();
        assert_eq!(receipt.value, "300".parse::<Decimal>().unwrap());
        assert_eq!(receipt.shared_by, vec!["Alice", "Sam"]);
    }

    #[test]
    fn test_duplicate_people() {
        let receipt = Receipt::parse_create_receipt("300,Alice,Sam,Alice");
        let _ = String::from("The provided names are duplicate.");
        assert!(matches!(
            receipt,
            Err(SplittingError::DuplicatePeopleError(_))
        ));
    }

    #[test]
    fn test_duplicate_people_cased() {
        let receipt = Receipt::parse_create_receipt("300,Alice,Sam,alice").unwrap();
        assert_eq!(receipt.value, dec![300]);
        assert_eq!(receipt.shared_by, vec!["Alice", "Sam", "alice"]);
    }

    #[test]
    fn test_aligning_to_extant_shared_people_fail() {
        let mut receipt = Receipt::parse_create_receipt("300,Alice,Sam,Samuel").unwrap();
        let val = receipt.align_to_shared_by("Al,S,S");
        let _ = "The abbreviation string: Al,S,S has duplicates.".to_string();
        assert!(matches!(val, Err(SplittingError::InvalidAbbreviation(_))));
    }

    #[test]
    fn test_aligning_to_extant_shared_people_pass() {
        let mut receipt = Receipt::parse_create_receipt("300,Alice,Sam,Samuel").unwrap();
        let val = receipt.align_to_shared_by("Al,S,Su").unwrap();
        assert_eq!(val, vec!["Alice", "Sam", "Samuel"]);
    }

    #[test]
    fn test_aligning_to_extant_shared_people_different_order_pass() {
        let mut receipt = Receipt::parse_create_receipt("300,Alice,Sam,Samuel").unwrap();
        let val = receipt.align_to_shared_by("Su,Al,S").unwrap();
        assert_eq!(val, vec!["Samuel", "Alice", "Sam"]);
    }

    #[test]
    fn match_person_by_abbreviation() {
        let mut receipt = Receipt::parse_create_receipt("300,Alice,Sam,Marshall").unwrap();
        receipt
            .parse_add_named_item("Caviar", "150,Al,S,M")
            .unwrap();
        receipt.parse_add_named_item("Drinks", "90,S,A").unwrap();
        assert_eq!(receipt.items[0].shared_by, vec!["Alice", "Sam", "Marshall"]);
        assert_eq!(receipt.items[1].shared_by, vec!["Sam", "Alice"]);

        let val = receipt.parse_add_named_item("More Drinks", "10,S,Sa,Al");
        let _ = format!("Sa maps to Sam, which has already been specified once.");
        assert!(matches!(val, Err(SplittingError::InvalidAbbreviation(_))));
    }

    // #[test]
    // fn add_tip_and_tax() {
    //     let mut receipt = Receipt::parse_create_receipt("300,Alice,Sam,Marshall").unwrap();
    //     receipt.parse_tip_or_tax(ItemType::Tip, "25").unwrap();
    //     receipt.parse_tip_or_tax(ItemType::Tax, "35").unwrap();
    //     assert_eq!(receipt.items[0].shared_by, vec!["Alice", "Sam", "Marshall"]);
    //     assert_eq!(receipt.items[0].value, ItemType::Tip);
    //     assert_eq!(receipt.items[0].value, Decimal::from_str("25").unwrap());
    //     assert_eq!(receipt.items[1].shared_by, vec!["Alice", "Sam", "Marshall"]);
    //     assert_eq!(receipt.items[1].item, ItemType::Tax);
    //     assert_eq!(receipt.items[1].value, Decimal::from_str("35").unwrap());
    // }
}
