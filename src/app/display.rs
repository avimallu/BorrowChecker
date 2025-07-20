use crate::app::{Route, RECEIPT_STATE};
use dioxus::prelude::*;

#[component]
pub fn DisplaySplits() -> Element {
    let nav = navigator();
    if let Some(receipt) = RECEIPT_STATE.read().as_ref() {
        let mut header = receipt.shared_by.clone();
        header.insert(0, "Item Name".into());
        header.push("Total".into());

        let (item_names, item_splits) = receipt.calculate_splits()?;

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
            document::Title { "BorrowChecker | View" }
            header { class: "hero is-small is-primary",
                div { class: "hero-body has-text-centered",
                    p { class: "title", "Here's your split!" }
                    p { class: "subtitle is-size-6",
                        "Balance leftover is distributed proportionally."
                    }
                }
            }
            div { class: "is-flex is-justify-content-center",
                div { class: "table-container ",
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
            footer { class: "hero is-small is-primary",
                div { class: "hero-body has-text-centered is-flex is-justify-content-center",
                    p { class: "subtitle is-size-7 mr-1", "Built with Rust & Dioxus | " }
                    p { class: "subtitle is-size-7 mr-1", "👾🤖👻 |" }
                    a {
                        class: "subtitle is-size-7 mr-1",
                        href: "https://avimallu.github.io",
                        "avimallu.github.io "
                    }
                }
            }
        }
    } else {
        nav.push(Route::CreateReceiptSplash);
        rsx! {}
    }
}
