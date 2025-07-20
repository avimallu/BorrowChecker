use crate::core::receipt::Receipt;
use dioxus::prelude::*;
use rust_decimal::prelude::*;

static CSS: Asset = asset!("/assets/bulma.css");

#[derive(Clone, PartialEq, Eq)]
enum AppState {
    CreatingReceipt,
    SplittingItems,
    DisplayingSplits,
}

#[component]
pub fn App() -> Element {
    let app_state = use_signal(|| AppState::CreatingReceipt);

    // Variables to create the essential values of the receipt
    let receipt_value: Signal<Option<Decimal>> = use_signal(|| None);
    let people_input: Signal<Vec<String>> = use_signal(|| vec!["".to_string()]);
    let people_list: Memo<Vec<String>> = use_memo(move || {
        people_input
            .read()
            .iter()
            .filter(|name| !name.is_empty())
            .cloned()
            .collect()
    });

    // Variables to input and handle receipt items
    let receipt: Signal<Option<Receipt>> = use_signal(|| None);

    rsx! {
        document::Stylesheet { href: CSS }
        section { class: "hero is-primary",
            div { class: "hero-body",
                p { class: "title", "Iron Abacus" }
                p { class: "subtitle", "A splitting helper" }
            }
        }
        if app_state() == AppState::CreatingReceipt {
            CreateReceipt {
                app_state,
                receipt_value,
                people_input,
                people_list,
                receipt,
            }
        } else if app_state() == AppState::SplittingItems {
            SplitReceipt {
                receipt_value,
                people_list,
                app_state,
                receipt,
            }
        } else {
            DisplayTable { app_state, receipt }
        }
    }
}

#[component]
fn DisplayTable(mut app_state: Signal<AppState>, mut receipt: Signal<Option<Receipt>>) -> Element {
    if let Some(valid_receipt) = receipt().as_ref() {
        let mut header = valid_receipt.shared_by.clone();
        header.insert(0, "Item Name".into());
        header.push("Total".into());

        let (item_names, item_splits) = valid_receipt.calculate_splits()?;

        let rows: Vec<Vec<String>> = item_names
            .into_iter()
            .zip(item_splits.into_iter())
            .map(|(item_name, splits)| {
                let mut splits_as_str: Vec<String> = splits.iter().map(|x| x.to_string()).collect();
                splits_as_str.insert(0, item_name.into());
                splits_as_str
            })
            .collect();

        rsx! {
            div { class: "table-container",
                table { class: "table",
                    thead {
                        tr {
                            for val in header.iter() {
                                th { scope: "col", "{val}" }
                            }
                        }
                    }
                    tbody {
                        for row in rows.iter() {
                            tr {
                                for (idx , val) in row.iter().enumerate() {
                                    if idx == 0 {
                                        th { scope: "row", "{val}" }
                                    } else {
                                        td { "{val}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! { "No table to show yet, bitches!" }
    }
}
