use crate::core::receipt::{Receipt, SplittingError};
use std::env;

// Super-basic parsing, advanced parsing packages are not needed
pub fn parse_args() -> Result<Receipt, SplittingError> {
    let args: Vec<String> = env::args().collect();

    // dbg!(&args);

    if args.len() < 2 {
        return Err(SplittingError::InvalidArgument(format!(
            "You have specified only the receipt's total value and people sharing it \
            but not any item within it to split. Please do so"
        )));
    } else {
        let mut receipt = Receipt::parse_create_receipt(&args[1])?;
        let mut curr_arg: Option<&str> = None;
        for (arg_idx, arg) in args[2..].iter().enumerate() {
            if curr_arg.is_none() {
                if arg.starts_with("--") {
                    curr_arg = Some(&arg[2..]);
                } else if arg.starts_with("-") {
                    curr_arg = Some(&arg[1..]);
                    continue;
                } else {
                    return Err(SplittingError::InvalidArgument(format!(
                        "Argument {} is expected (in this case) to be an item name, \
                        and must be prefixed with a dash (-) or a double dash (--). Currently, \
                        it is {}",
                        arg_idx + 1,
                        arg
                    )));
                }
            } else {
                receipt.parse_add_named_item(curr_arg.unwrap(), arg)?;
                curr_arg = None
            }
        }
        Ok(receipt)
    }
}
