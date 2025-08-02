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
    InvalidIndexError(String),
    NotProportionallySplittableError(String),
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
            Self::InvalidIndexError(msg) => write!(f, "{}", msg),
            Self::NotProportionallySplittableError(msg) => write!(f, "{}", msg),
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

    // Obtain a single vector with the exact splits
    fn calculate_receipt_proportions(&self) -> Vec<Decimal> {
        let items = self.items.iter().filter(|&x| !x.is_prop_dist);

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

    pub fn calculate_item_share_ratio_by_proportion(
        &self,
        shared_by: &Vec<String>,
        value: Decimal,
    ) -> Vec<Decimal> {
        // Align the proportional splits to the current shared_by ratios
        let pre_prop_splits: Vec<Decimal> = self
            .calculate_receipt_proportions()
            .iter()
            .zip(self.shared_by.iter())
            .filter(|(_, person)| shared_by.contains(*person))
            .map(|(split, _)| split.clone())
            .collect();

        let pre_prop_split_total: Decimal = pre_prop_splits.iter().sum();
        let proportional_splits: Vec<Decimal> = pre_prop_splits
            .iter()
            .map(|x| x / pre_prop_split_total)
            .collect();
        return proportional_splits
            .iter()
            .map(|ratio| (ratio * value).round_dp(2))
            .collect();
    }

    pub fn add_item_split_by_proportion(
        &mut self,
        value: Decimal,
        name: String,
        shared_by: Vec<String>,
    ) -> Result<&mut Self, SplittingError> {
        if shared_by.len() == 0 {
            return Err(SplittingError::InvalidShareConfiguration(format!(
                "Number of people sharing this receipt cannot be zero."
            )));
        }
        if name.is_empty() {
            return Err(SplittingError::InvalidFieldError(
                "Item name cannot be empty".into(),
            ));
        }

        let share_ratio = self.calculate_item_share_ratio_by_proportion(&shared_by, value);

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
                // If the person is sharing the item, specify the share as the person's share
                // divided by the item's total shares. This means that an item can be shared
                // proportional to other costs by fewer people than those present in the receipt.
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
            let overall_prop = self.calculate_receipt_proportions();
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

    // A ReceiptItem can be split proportionally iff at least ONE
    // other receipt item is not split by proportion.
    fn is_proportionally_splittable(&self, index: usize) -> bool {
        let boo: Vec<bool> = self
            .items
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx != index)
            .map(|(_, x)| x.is_prop_dist)
            .collect();

        return !boo.into_iter().min().unwrap_or(true);
    }

    pub fn recalculate_proportions(&mut self) {
        let mut item_share_ratios: Vec<Vec<Decimal>> = Vec::new();
        for item in self.items.iter().filter(|x| x.is_prop_dist) {
            item_share_ratios
                .push(self.calculate_item_share_ratio_by_proportion(&item.shared_by, item.value));
        }
        for (item, share_ratio) in self
            .items
            .iter_mut()
            .filter(|x| x.is_prop_dist)
            .zip(item_share_ratios.into_iter())
        {
            item.share_ratio = share_ratio
        }
    }

    pub fn update_item_at_index(
        &mut self,
        idx: usize,
        value: Option<Decimal>,
        name: Option<String>,
        shared_by: Option<Vec<String>>,
        is_prop_dist: Option<bool>,
    ) -> Result<(), SplittingError> {
        let is_proportionally_splittable = self.is_proportionally_splittable(idx);

        if let Some(receipt_item) = self.items.get_mut(idx) {
            // We could use `map` here to be succinct, but that's supposed to be an
            // anti-pattern? "Don't use map for its side effect".
            if let Some(value_) = value {
                receipt_item.value = value_
            }
            if let Some(name_) = name {
                receipt_item.name = name_
            }

            // Learning: Decimal, bool and String implement copy, but Vec<String>
            // does not, that is why a manual `.clone()` is required here.
            if let Some(shared_by_) = shared_by.clone() {
                receipt_item.shared_by = shared_by_;
                receipt_item.share_ratio = vec![Decimal::ONE; receipt_item.shared_by.len()];
            }
            if let Some(is_prop_dist_) = is_prop_dist {
                if is_prop_dist_ && is_proportionally_splittable {
                    // Setting is_prop_dist to true when possible
                    receipt_item.is_prop_dist = true;
                    receipt_item.shared_by = self.shared_by.clone();
                } else if !is_prop_dist_ {
                    // Setting is_prop_dist to false and sharing it across all people
                    receipt_item.is_prop_dist = false;
                    receipt_item.shared_by = self.shared_by.clone();
                    receipt_item.share_ratio = vec![Decimal::ONE; self.shared_by.len()];
                } else {
                    return Err(SplittingError::NotProportionallySplittableError(
                        "There aren't enough items left to split proportionally on".into(),
                    ));
                }
            }
        } else {
            return Err(SplittingError::InvalidIndexError(
                "Provide index is outside the range of items present in the receipt".into(),
            ));
        }

        // Setting the item as (not) proportional means that it is (no longer) determining
        // proportional splits for other items. Therefore, this proportion needs to be recalculated.

        // This is true for any change except for a change in name of an underlying item, as long
        // as proportional items exist.

        // When the last proportional item is changed to being non-proportional, the adjustment
        // to its own value is already made in the `if !is_prop_dist` branch above, so no further
        // changes need to be made.

        if self.items.iter().filter(|x| x.is_prop_dist).count() > 0
            && (value.is_some() || shared_by.is_some() || is_prop_dist.is_some())
        {
            self.recalculate_proportions()
        }

        Ok(())
    }

    // Removing, as opposed to updating, is a far simpler operation - just remove the
    // index specified, and update all other values that depend on proportion. Voila!
    pub fn remove_item_at_index(&mut self, idx: usize) -> Result<(), SplittingError> {
        let proportional_count = self.items.iter().filter(|x| x.is_prop_dist).count();

        if idx >= self.items.len() {
            return Err(SplittingError::InvalidIndexError(
                "Provided index is out of bounds".to_string(),
            ));
        }
        // Disallow removal of the last proportional item since the rest depend on it
        else if self.items.iter().filter(|x| !x.is_prop_dist).count() == 1
            && proportional_count > 0
            && !self.items.get(idx).unwrap().is_prop_dist
        {
            return Err(SplittingError::InvalidIndexError(
                "The last non-proportional item cannot be removed when there are proportional items in the receipt.".into()
            ));
        }

        self.items.remove(idx);

        if proportional_count > 0 {
            self.recalculate_proportions();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::receipt::{Receipt, SplittingError};
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

    fn proportional_receipt_helper() -> Result<Receipt, SplittingError> {
        let mut receipt = Receipt::new(dec![300], vec!["Alice", "Bob", "Marshall"])?;
        receipt
            .add_item_split_by_ratio(
                dec![30],
                "Hearty Burger".into(),
                utils::strs_to_strings(vec!["Alice"]),
                None,
            )?
            .add_item_split_by_ratio(
                dec![60],
                "Unhealthy Burger".into(),
                utils::strs_to_strings(vec!["Bob"]),
                None,
            )?
            .add_item_split_by_ratio(
                dec![15],
                "Vegan Salad".into(),
                utils::strs_to_strings(vec!["Marshall"]),
                None,
            )?
            .add_item_split_by_proportion(
                dec![50],
                "Tax".into(),
                utils::strs_to_strings(vec!["Alice", "Bob"]),
            )?
            .add_item_split_by_proportion(
                dec![50],
                "Tip".into(),
                utils::strs_to_strings(vec!["Bob", "Marshall"]),
            )?;
        Ok(receipt)
    }

    #[test]
    fn test_adding_by_proportions() {
        let receipt = proportional_receipt_helper().unwrap();
        assert_eq!(
            receipt.items[3].share_ratio,
            f64s_to_decimals(&[16.67, 33.33])
        );
        assert_eq!(
            receipt.items[4].share_ratio,
            f64s_to_decimals(&[40.0, 10.0])
        );
        println!("{:#?}", receipt);
    }

    #[test]
    fn test_updated_items() {
        let mut receipt = proportional_receipt_helper().unwrap();
        // At this point, the receipt is:
        // #     Alice   Bob   Marshall   Is Prop?
        // 0        30
        // 1               60
        // 2                         15
        // 3      16.67 33.33                    x
        // 4               40      37.5          x
        let _ = receipt.update_item_at_index(2, Some(dec![30]), None, None, None);

        // At this point, the receipt should be:
        // #     Alice   Bob   Marshall   Is Prop?
        // 0        30
        // 1               60
        // 2                         30
        // 3      16.67 33.33                    x
        // 4            33.33     16.67          x
        assert_eq!(
            receipt.items[3].share_ratio,
            f64s_to_decimals(&[16.67, 33.33])
        );
        assert_eq!(
            receipt.items[4].share_ratio,
            f64s_to_decimals(&[33.33, 16.67])
        );

        let _ = receipt.update_item_at_index(2, None, Some("Vegan Air".into()), None, None);

        assert_eq!(
            receipt.items[3].share_ratio,
            f64s_to_decimals(&[16.67, 33.33])
        );
        assert_eq!(
            receipt.items[4].share_ratio,
            f64s_to_decimals(&[33.33, 16.67])
        );
        assert_eq!(receipt.items[2].name, "Vegan Air");

        let _ = receipt.update_item_at_index(
            1,
            None,
            None,
            Some(utils::strs_to_strings(vec!["Bob", "Marshall"])),
            None,
        );
        // At this point, the receipt should be:
        // #     Alice   Bob   Marshall   Is Prop?
        // 0        30
        // 1               30        30
        // 2                         30
        // 3        25     25                    x
        // 4            16.67     33.33          x

        assert_eq!(receipt.items[1].shared_by, vec!["Bob", "Marshall"]);
        assert_eq!(
            receipt.items[3].share_ratio,
            f64s_to_decimals(&[25.0, 25.0])
        );
        assert_eq!(
            receipt.items[4].share_ratio,
            f64s_to_decimals(&[16.67, 33.33])
        );

        let _ = receipt.update_item_at_index(4, None, None, None, Some(false));
        // At this point, the receipt should be:
        // #     Alice   Bob  Marshall   Is Prop?
        // 1        30
        // 2              30        30
        // 3                        30
        // 4        25    25                    x
        // 5     13.33  13.33    13.33          x

        assert_eq!(
            receipt.items[3].share_ratio,
            f64s_to_decimals(&[25.0, 25.0])
        );
        assert_eq!(
            receipt.items[4].share_ratio,
            f64s_to_decimals(&[1.0, 1.0, 1.0])
        );

        for i in 0..3 {
            let _ = receipt.update_item_at_index(i, None, None, None, Some(true));
        }
        // This should fail since this is the last non-proportional item (3 is already proportional)
        let result = receipt.update_item_at_index(4, None, None, None, Some(true));
        assert!(matches!(
            result,
            Err(SplittingError::NotProportionallySplittableError(_))
        ));

        // Should work fine now!
        let _ = receipt.update_item_at_index(3, None, None, None, Some(false));
        let result = receipt.update_item_at_index(4, None, None, None, Some(true));
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_removing_items() {
        let mut receipt_1 = proportional_receipt_helper().unwrap();
        let mut receipt_2 = proportional_receipt_helper().unwrap();
        // Starting point of the receipt is
        // #     Alice   Bob  Marshall   Is Prop?
        // 0        30
        // 1              60
        // 2                        15
        // 3        25    25                    x
        // 4              40        10          x
        let _ = receipt_1.remove_item_at_index(2);
        assert_eq!(
            receipt_1.items[3].share_ratio,
            f64s_to_decimals(&[50.0, 0.0])
        );

        let _ = receipt_2.update_item_at_index(
            1,
            None,
            None,
            Some(utils::strs_to_strings(vec!["Alice", "Bob", "Marshall"])),
            None,
        );
        // At this point, the receipt is:
        // #     Alice   Bob  Marshall   Is Prop?
        // 0        30
        // 1        20    20        20
        // 2                        15
        // 3      37.5  12.5                    x
        // 4            12.5      37.5          x
        assert_eq!(
            receipt_2.items[3].share_ratio,
            f64s_to_decimals(&[35.71, 14.29])
        );
        assert_eq!(
            receipt_2.items[4].share_ratio,
            f64s_to_decimals(&[18.18, 31.82])
        );

        let _ = receipt_2.remove_item_at_index(1);
        // At this point, the receipt should be:
        // #     Alice   Bob  Marshall   Is Prop?
        // 0        30
        // 1                        15
        // 2        50.                         x
        // 3                        50          x
        assert_eq!(
            receipt_2.items[2].share_ratio,
            f64s_to_decimals(&[50.0, 0.0])
        );
        assert_eq!(
            receipt_2.items[3].share_ratio,
            f64s_to_decimals(&[0.0, 50.0])
        );
    }
}
