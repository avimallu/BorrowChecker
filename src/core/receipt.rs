use crate::utils;
use rust_decimal::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

type Person = String;
const LEFTOVER_ITEM_NAME: &'static str = "<leftover>";
const TOTAL_ITEM_NAME: &'static str = "<total>";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub value: Decimal,
    pub shared_by: Vec<Person>,
    pub mapped_abbreviations: HashMap<Person, String>,
    pub items: Vec<ReceiptItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptItem {
    pub value: Decimal,
    pub name: String,
    pub shared_by: Vec<Person>,
    pub share_ratio: Vec<Decimal>,
    // is_proportionally_distributed
    pub is_prop_dist: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SplittingError {
    DuplicatePeopleError(String),
    NotEnoughPeopleError(String),
    InvalidShareConfiguration(String),
    InvalidFieldError(String),
    InvalidAbbreviation(String),
    InternalError(String),
    ItemTotalExceedsReceiptTotal(String),
    DecimalParsingError(String),
    InvalidArgument(String),
}

impl From<rust_decimal::Error> for SplittingError {
    fn from(e: rust_decimal::Error) -> SplittingError {
        SplittingError::DecimalParsingError(e.to_string())
    }
}

// Required for main and Box<dyn std::error::Error>> returns to not complain
impl Error for SplittingError {}

impl fmt::Display for SplittingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePeopleError(msg) => write!(f, "{}", msg),
            Self::NotEnoughPeopleError(msg) => write!(f, "{}", msg),
            Self::InvalidShareConfiguration(msg) => write!(f, "{}", msg),
            Self::InvalidFieldError(msg) => write!(f, "{}", msg),
            Self::InvalidAbbreviation(msg) => write!(f, "{}", msg),
            Self::InternalError(msg) => write!(f, "{}", msg),
            Self::ItemTotalExceedsReceiptTotal(msg) => write!(f, "{}", msg),
            Self::DecimalParsingError(msg) => write!(f, "{}", msg),
            Self::InvalidArgument(msg) => write!(f, "{}", msg),
        }
    }
}

impl Receipt {
    // Creates a new Receipt with just the (total) value and the people sharing it.
    // Mapping is defaulted to a new, empty HashMap.
    // Items is an empty vector.
    pub fn new(value: Decimal, shared_by: Vec<&str>) -> Result<Receipt, SplittingError> {
        utils::is_string_vec_unique(
            &shared_by,
            SplittingError::DuplicatePeopleError(
                "The list of people sharing the receipt is duplicated. Please disambiguate.".into(),
            ),
        )?;
        utils::is_vec_len_gt_1(
            &shared_by,
            SplittingError::NotEnoughPeopleError(
                "A receipt has to be shared by at least 2 people.".into(),
            ),
        )?;

        Ok(Receipt {
            value,
            shared_by: shared_by.iter().map(|&x| x.to_string()).collect(),
            mapped_abbreviations: HashMap::new(),
            items: vec![],
        })
    }

    pub fn add_item_split_by_ratio(
        &mut self,
        value: Decimal,
        name: String,
        shared_by: Vec<String>,
        share_ratio: Option<Vec<Decimal>>,
    ) -> Result<&mut Self, SplittingError> {
        let share_ratio = share_ratio.unwrap_or(vec![Decimal::ONE; shared_by.len()]);

        if shared_by.len() != share_ratio.len() {
            return Err(SplittingError::InvalidShareConfiguration(format!(
                "Length mismatch: people sharing {} and the ratios of the shares {} have differing lengths.",
                shared_by.len(),
                share_ratio.len()
            )));
        } else if shared_by.len() == 0 {
            return Err(SplittingError::NotEnoughPeopleError(format!(
                "The number of people sharing the item {} is {}. It must be shared by at least 1 person.",
                name,
                shared_by.len()
            )));
        }

        if name.is_empty() {
            return Err(SplittingError::InvalidFieldError(
                "Item name cannot be empty".into(),
            ));
        }

        self.items.push(ReceiptItem {
            value,
            name,
            shared_by,
            share_ratio,
            is_prop_dist: false,
        });
        Ok(self)
    }

    // Obtain a single vector with the exact splits, with or without items with
    // the is_prop_dist attribute as true
    fn calculate_overall_proportion(&self, remove_proportional_items: bool) -> Vec<Decimal> {
        let items = self
            .items
            .iter()
            .filter(|&x| !remove_proportional_items || !x.is_prop_dist);

        let mut receipt_split: Vec<Decimal> = vec![Decimal::ZERO; self.shared_by.len()];
        for item in items {
            let denominator: Decimal = item.share_ratio.iter().sum();

            // Split each item.value proportional to the share ratios of the people sharing
            // the item, in the order in which these people appear in self.shared_by
            let item_split: Vec<Decimal> = self
                .shared_by
                .iter()
                .map(|person| {
                    item.shared_by
                        .iter()
                        .zip(item.share_ratio.iter())
                        // The first match is all that is required because other operations guarantee
                        // that duplicate names do not exist in either self.shared_by or item.shared_by
                        .find(|&(sharer, _)| *person == *sharer)
                        .map(|(_, &numerator)| (numerator / denominator * item.value))
                        .unwrap_or_else(|| Decimal::ZERO)
                })
                .collect();

            for (idx, split) in item_split.iter().enumerate() {
                receipt_split[idx] += split
            }
        }
        receipt_split
    }

    pub fn add_item_split_by_proportion(
        &mut self,
        value: Decimal,
        name: String,
        shared_by: Vec<String>,
    ) -> Result<&mut Self, SplittingError> {
        if shared_by.len() == 0 {
            return Err(SplittingError::NotEnoughPeopleError(format!(
                "The number of people sharing the item {} is currently {}. It must be shared by at least 1 person.",
                name,
                shared_by.len()
            )));
        } else if name.is_empty() {
            return Err(SplittingError::InvalidFieldError(
                "Item name cannot be empty".into(),
            ));
        }

        // These vectors are aligned with the order of self.shared_by
        let pre_prop_splits = self.calculate_overall_proportion(true);
        let pre_prop_split_total: Decimal = pre_prop_splits.iter().sum();
        let pre_prop_ratios: Vec<Decimal> = pre_prop_splits
            .iter()
            .map(|x| x / pre_prop_split_total)
            .collect();

        // This vector's length will be identical to shared_by
        let share_ratio: Vec<Decimal> = shared_by
            .iter()
            .map(|sharer| {
                pre_prop_ratios
                    .iter()
                    .zip(self.shared_by.iter())
                    .find(|&(_, person)| *sharer == *person)
                    .map(|(&ratio, _)| ratio * value)
                    .ok_or_else(|| {
                        SplittingError::InternalError(format!(
                            "The sharer {} was not found among the original sharers.",
                            sharer,
                        ))
                    })
            })
            .collect::<Result<_, _>>()?;

        self.items.push(ReceiptItem {
            value,
            name,
            shared_by,
            share_ratio,
            is_prop_dist: true,
        });
        Ok(self)
    }

    pub fn get_itemized_total_and_leftover(&self) -> (Decimal, Decimal) {
        let itemized_total: Decimal = self.items.iter().map(|x| x.value).sum();
        let leftover_amount: Decimal = self.value - itemized_total;
        (itemized_total, leftover_amount)
    }

    // Get a vector of item names (including leftovers and totals), as well as the splits
    // by each item so that they can be eventually displayed in a table easily, or used
    // for any other purpose.
    pub fn calculate_splits(&self) -> Result<(Vec<&str>, Vec<Vec<Decimal>>), SplittingError> {
        // let itemized_total: Decimal = self.items.iter().map(|x| x.value).sum();
        // let leftover_amount: Decimal = self.value - itemized_total;
        let (itemized_total, leftover_amount) = self.get_itemized_total_and_leftover();
        match leftover_amount.cmp(&Decimal::ZERO) {
            // There is a problem only if the leftover amount is negative
            Ordering::Greater | Ordering::Equal => {}
            Ordering::Less => {
                return Err(SplittingError::ItemTotalExceedsReceiptTotal(format!(
                    "The itemized total amount {} exceeds the receipt's total amount {} by {}",
                    itemized_total, self.value, leftover_amount
                )));
            }
        };

        let mut all_splits: Vec<Vec<Decimal>> = Vec::new();

        // Refactor needed - Receipts are short lived, so there is no point in
        // converting between ReceiptItem.shared_by and Receipt.shared_by - just
        // store shared_by in the same order as the receipt and display to the
        // user all the shared_by values that don't have 0 share ratio.
        for item in self.items.iter() {
            let mut splits: Vec<Decimal> = self
                .shared_by
                .iter()
                .map(|x| match item.shared_by.iter().position(|name| name == x) {
                    Some(pos) => (item.value * item.share_ratio[pos]
                        / item.share_ratio.iter().sum::<Decimal>())
                    .round_dp(2),
                    None => Decimal::ZERO.round_dp(2),
                })
                .collect();
            splits.push(item.value);
            all_splits.push(splits);
        }

        let mut item_names: Vec<&str> = self.items.iter().map(|x| x.name.as_str()).collect();

        // Add unaccounted item, if present
        if leftover_amount > Decimal::ZERO {
            let overall_prop = self.calculate_overall_proportion(true);
            let overall_prop_sum: Decimal = overall_prop.iter().sum();
            let mut splits: Vec<Decimal> = overall_prop
                .iter()
                .map(|x| (x * leftover_amount / overall_prop_sum).round_dp(2))
                .collect();
            splits.push(leftover_amount);
            all_splits.push(splits);
            item_names.push(LEFTOVER_ITEM_NAME);
        }

        // Range from 0 to len + 1 to account for total added at the end of each item's share
        let total_split: Vec<Decimal> = (0..(self.shared_by.len() + 1))
            .map(|i| all_splits.iter().map(|v| v[i]).sum::<Decimal>().round_dp(2))
            .collect();
        all_splits.push(total_split);
        item_names.push(TOTAL_ITEM_NAME);

        Ok((item_names, all_splits))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::receipt::Receipt;
    use crate::utils;
    use rust_decimal::prelude::*;

    fn f64s_to_decimals(values: &[f64]) -> Vec<Decimal> {
        values
            .iter()
            .map(|x| Decimal::from_f64(*x).unwrap())
            .collect()
    }

    #[test]
    fn test_calculate_splits() {
        let mut receipt = Receipt::new(dec![300], vec!["Alice", "Bob", "Marshall"]).unwrap();
        let _ = receipt
            .add_item_split_by_ratio(
                dec![200],
                "Food".into(),
                utils::strs_to_strings(vec!["Alice", "Bob", "Marshall"]),
                None,
            )
            .unwrap();
        let _ = receipt
            .add_item_split_by_ratio(
                dec![50],
                "Drinks".into(),
                utils::strs_to_strings(vec!["Alice", "Bob"]),
                None,
            )
            .unwrap();
        let (_, expected_splits) = receipt.calculate_splits().unwrap();
        let actual_splits: Vec<Vec<Decimal>> = vec![
            f64s_to_decimals(&[66.67, 66.67, 66.67, 200.0]),
            f64s_to_decimals(&[25.0, 25.0, 0.0, 50.0]),
            f64s_to_decimals(&[18.33, 18.33, 13.33, 50.0]),
            f64s_to_decimals(&[110.0, 110.0, 80.0, 300.0]),
        ];
        assert_eq!(expected_splits, actual_splits);
    }
}
