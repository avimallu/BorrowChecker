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
                div { class: "container",
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
    let disable_proportional_button: String = match (*RECEIPT_STATE.read())
        .as_ref()
        .unwrap()
        .is_proportionally_splittable(item_idx)
    {
        false => "true".into(),
        true => "false".into(),
    };
    let disable_removal_button: String = match (*RECEIPT_STATE.read())
        .as_ref()
        .unwrap()
        .is_removable(item_idx)
    {
        false => "true".into(),
        true => "false".into(),
    };

    let (item_name, item_value, item_shared_by, item_is_prop_dist) = (&RECEIPT_STATE.read())
        .as_ref()
        .and_then(|r| r.items.get(item_idx))
        .map(|item| {
            (
                item.name.clone(),
                item.value, //.clone(),
                item.shared_by.clone(),
                item.is_prop_dist, //.clone(),
            )
        })
        .unwrap_or_default();

    let item_value = if item_value > Decimal::ZERO {
        item_value.to_string()
    } else {
        "-".to_string()
    };

    if true {
        rsx! {
            div { class: "columns is-mobile is-1 is-vcentered",
                div { class: "column is-5",
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
                div { class: "column is-3",
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
                        placeholder: "$",
                    }
                }
                div { class: "column is-2",
                    button {
                        class: if item_is_prop_dist { "button is-dark is-info is-fullwidth" } else { "button is-dark is-outlined is-info is-fullwidth" },
                        key: "item_proportional_button_{item_idx}",
                        disabled: disable_proportional_button,
                        onclick: move |_| {
                            if let Some(receipt) = RECEIPT_STATE.write().as_mut() {
                                receipt
                                    .update_item_at_index(
                                        item_idx,
                                        None,
                                        None,
                                        None,
                                        Some(!item_is_prop_dist.clone()),
                                    )
                                    .unwrap();
                            }
                        },
                        Icon {
                            width: 24,
                            height: 24,
                            fill: "white",
                            icon: ld_icons::LdPercent,
                        }
                    }
                }
                div { class: "column is-2",
                    button {
                        class: "button is-dark is-danger is-fullwidth",
                        key: "item_delete_button_{item_idx}",
                        disabled: disable_removal_button,
                        onclick: move |_| {
                            if let Some(receipt) = RECEIPT_STATE.write().as_mut() {
                                receipt.remove_item_at_index(item_idx).unwrap();
                            }
                        },
                        Icon {
                            width: 24,
                            height: 24,
                            fill: "white",
                            icon: ld_icons::LdCircleX,
                        }
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
                                if let Some(receipt) = RECEIPT_STATE.write().as_mut() {
                                    let mut new_item_shared_by = receipt.items[item_idx].shared_by.clone();
                                    if new_item_shared_by.contains(&person) && new_item_shared_by.len() > 1 {
                                        let new_item_shared_by = new_item_shared_by
                                            .iter()
                                            .filter(|&x| *x != person)
                                            .cloned()
                                            .collect();
                                        receipt
                                            .update_item_at_index(
                                                item_idx,
                                                None,
                                                None,
                                                Some(new_item_shared_by),
                                                None,
                                            )
                                            .unwrap();
                                    } else if !new_item_shared_by.contains(&person) {
                                        new_item_shared_by.push(person.clone());
                                        receipt
                                            .update_item_at_index(
                                                item_idx,
                                                None,
                                                None,
                                                Some(new_item_shared_by),
                                                None,
                                            )
                                            .unwrap();
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
