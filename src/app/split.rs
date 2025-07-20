use crate::app::{Route, RECEIPT_STATE};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons;
use dioxus_free_icons::Icon;
use rust_decimal::prelude::*;

#[component]
pub fn SplitUI() -> Element {
    let nav = navigator();
    if let Some(receipt) = RECEIPT_STATE.read().as_ref() {
        let (_, balance) = receipt.get_itemized_total_and_leftover();
        let item_count = receipt.items.len();
        rsx! {
            document::Title { "BorrowChecker | Split" }
            header { class: "hero is-small is-primary",
                div { class: "hero-body has-text-centered",
                    ColorBalanceTitle { balance }
                }
            }
            div { class: "section",
                div { class: "container is-fluid",
                    for item_idx in 0..item_count {
                        SplitItemUI { item_idx }
                    }
                }
                div { class: "is-flex is-justify-content-center",
                    div { class: "buttons",
                        div {
                            button {
                                class: "button is-primary is-dark",
                                key: "item_add_button",
                                onclick: move |_| {
                                    if let Some(r) = RECEIPT_STATE.write().as_mut() {
                                        let people_list = r.shared_by.clone();
                                        r.add_item_split_by_ratio(
                                                Decimal::ZERO,
                                                format!("Item {}", item_count + 1),
                                                people_list,
                                                None,
                                            )
                                            .unwrap();
                                    }
                                },
                                Icon {
                                    width: 24,
                                    height: 24,
                                    fill: "white",
                                    icon: ld_icons::LdBookPlus,
                                }
                                span { class: "ml-2", "Add Item" }

                            }
                        }
                        div {
                            if receipt.items.len() > 0 && receipt.items.iter().all(|x| x.value > Decimal::ZERO)
                                && receipt.calculate_splits().is_ok()
                            {
                                button {
                                    class: "button is-link is-dark",
                                    key: "show_calculated_table",
                                    onclick: move |_| {
                                        nav.push(Route::DisplaySplits);
                                    },
                                    Icon {
                                        width: 24,
                                        height: 24,
                                        fill: "white",
                                        icon: ld_icons::LdScale,
                                    }
                                    span { class: "ml-2", "Show Splits" }
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

#[component]
fn ColorBalanceTitle(balance: Decimal) -> Element {
    rsx! {
        if balance > Decimal::ZERO {
            p { class: "title has-text-dark is-size-4", "+{balance}" }
            p { class: "subtitle is-size-5", "left to balance" }
        } else if balance < Decimal::ZERO {
            p { class: "title has-text-danger is-size-4", "Remaining: {balance}" }
            p { class: "subtitle is-size-6", "Item total exceeds receipt total." }
        } else {
            p { class: "title has-text-link  is-size-4", "0" }
            p { class: "subtitle is-size-6", "Perfectly balanced, as all things should be." }
        }
    }
}

#[component]
fn SplitItemUI(item_idx: usize) -> Element {
    let people_list = (*RECEIPT_STATE.read()).as_ref().unwrap().shared_by.clone();

    let (item_name, item_value, item_shared_by) = &RECEIPT_STATE
        .read()
        .as_ref()
        .and_then(|r| r.items.get(item_idx))
        .map(|item| {
            (
                item.name.clone(),
                item.value.clone(),
                item.shared_by.clone(),
            )
        })
        .unwrap_or_default();

    let item_value = if item_value > &Decimal::ZERO {
        item_value.to_string()
    } else {
        "-".to_string()
    };

    if true {
        rsx! {
            div { class: "columns is-mobile",
                div { class: "column is-two-thirds",
                    input {
                        class: "input is-primary",
                        key: "item_input_name_{item_idx}",
                        r#type: "text",
                        value: "{item_name}",
                        oninput: move |evt| {
                            if let Some(r) = RECEIPT_STATE.write().as_mut() {
                                if let Some(item) = r.items.get_mut(item_idx) {
                                    item.name = evt.value();
                                }
                            }
                        },
                        placeholder: "item name",
                    }
                }
                div { class: "column is-one-third",
                    input {
                        class: "input is-primary",
                        key: "item_input_value_{item_idx}",
                        min: "0.00",
                        step: "0.01",
                        inputmode: "decimal",
                        required: "true",
                        r#type: "number",
                        value: "{item_value}",
                        oninput: move |evt| {
                            if let Some(r) = RECEIPT_STATE.write().as_mut() {
                                if let Some(item) = r.items.get_mut(item_idx) {
                                    if let Ok(valid_decimal) = evt.value().parse::<Decimal>() {
                                        item.value = valid_decimal;
                                    } else {
                                        item.value = Decimal::ZERO;
                                    }
                                }
                            }
                        },
                        placeholder: "amount",
                    }
                }
            }
            div { class: "buttons",
                for (person_idx , person) in people_list.clone().into_iter().enumerate() {
                    div {
                        button {
                            class: if item_shared_by.contains(&person) { "button is-primary is-dark is-fullwidth" } else { "button is-primary is-outlined is-dark is-fullwidth" },
                            key: "item_{item_idx}_person_{person_idx}",
                            onclick: move |_| {
                                if let Some(r) = RECEIPT_STATE.write().as_mut() {
                                    if let Some(item) = r.items.get_mut(item_idx) {
                                        if item.shared_by.contains(&person) && item.shared_by.len() > 1 {
                                            let shared_by_idx = item
                                                .shared_by
                                                .iter()
                                                .position(|name| *name == *person)
                                                .unwrap();
                                            item.shared_by.remove(shared_by_idx);
                                            item.share_ratio.remove(shared_by_idx);
                                        } else {
                                            item.shared_by.push(person.clone());
                                            item.share_ratio.push(Decimal::ONE);
                                        }
                                    }
                                }
                            },
                            "{person}"
                        }
                    }
                }
            }
            hr {}
        }
    } else {
        rsx! { "Unhandled error 2" }
    }
}
