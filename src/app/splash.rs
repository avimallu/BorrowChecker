use crate::app::storage::use_persistent;
use crate::app::{Route, RECEIPT_STATE};
use crate::core::receipt::Receipt;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons;
use dioxus_free_icons::Icon;
use rust_decimal::prelude::*;

#[component]
pub fn CreateReceiptSplash() -> Element {
    let receipt_value: Signal<Option<Decimal>> = use_signal(|| None);
    let people_input: Signal<Vec<String>> = use_signal(|| vec!["".to_string()]);
    let people_list: Memo<Vec<String>> = use_memo(move || {
        people_input
            .read()
            .iter()
            .filter(|&name| !name.is_empty())
            .cloned()
            .collect()
    });

    rsx! {
        document::Title { "BorrowChecker | Create" }
        header { class: "hero is-small is-primary",
            div { class: "hero-body has-text-centered",
                p { class: "title", "BorrowChecker" }
                p { class: "subtitle is-size-6", "A utility to determine how who owes what" }
            }
        }
        div { class: "section is-small",
            ReceiptValue { receipt_value }
            hr {}
            ReceiptPeopleList { people_input }
        }
        div { class: "section",
            SubmitReceipt { receipt_value, people_list }
            RetrieveCache { people_input }
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
}

#[component]
fn ReceiptValue(mut receipt_value: Signal<Option<Decimal>>) -> Element {
    rsx! {
        div { class: "container is-fluid",
            input {
                class: "input is-primary",
                min: 0.01,
                placeholder: "Enter receipt total",
                step: "0.01",
                required: "true",
                r#type: "number",
                inputmode: "decimal",
                value: "{receipt_value.read().map_or_else(String::new, |d| d.to_string())}",
                oninput: move |event| {
                    if let Ok(value) = event.value().parse::<Decimal>() {
                        receipt_value.set(Some(value));
                    } else {
                        receipt_value.set(None);
                    }
                },
            }
        }
    }
}

#[component]
fn ReceiptPeopleList(mut people_input: Signal<Vec<String>>) -> Element {
    rsx! {
        div { class: "container is-fluid",
            for (idx , person) in people_input().iter().enumerate() {
                div { key: "people_input_div_{idx}", class: "columns is-mobile",

                    div { class: "column is-9",
                        input {
                            class: "input is-primary",
                            key: "people_input_text_{idx}",
                            r#type: "text",
                            minlength: 0,
                            placeholder: "Enter person {idx+1} name",
                            value: "{person}",
                            oninput: move |evt| { people_input.with_mut(|list| list[idx] = evt.value()) },
                        }
                    }

                    div { class: "column is-1 is-narrow",
                        // Render ❌ only one of the two conditions is true:
                        // 1) The current item is not the last item in the list and
                        // 2) There are more than 2 items in the list
                        //
                        // Otherwise, always render ➕ unless the input is an empty string.
                        if people_input.len() > 1 && (idx + 1) != people_input.len() {
                            button {
                                class: "button is-danger is-dark is-rounded",
                                key: "people_input_remove_button_{idx}",
                                onclick: move |_| {
                                    people_input.with_mut(|list| list.remove(idx));
                                },
                                Icon {
                                    width: 24,
                                    height: 24,
                                    fill: "white",
                                    icon: ld_icons::LdCircleX,
                                }
                            }
                        } else if person != "" {
                            button {
                                class: "button is-primary is-dark is-rounded",
                                key: "people_input_add_button_{idx}",
                                onclick: move |_| {
                                    people_input.push("".to_string());
                                },
                                Icon {
                                    width: 24,
                                    height: 24,
                                    fill: "white",
                                    icon: ld_icons::LdPlus,
                                }
                            }
                        } else {

                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SubmitReceipt(
    receipt_value: Signal<Option<Decimal>>,
    people_list: Memo<Vec<String>>,
) -> Element {
    let nav = navigator();
    if !receipt_value().is_none() && people_list.read().len() > 0 {
        let generated_receipt = Receipt::new(
            receipt_value().unwrap(),
            people_list().iter().map(|x| x.as_str()).collect(),
        );
        match generated_receipt {
            Ok(valid_receipt) => {
                rsx! {
                    div { class: "container is-fluid",
                        button {
                            class: "button is-success is-dark is-large is-fullwidth",
                            key: "submit_receipt",
                            onclick: move |_| {
                                set_people(people_list().clone());
                                *RECEIPT_STATE.write() = Some(valid_receipt.clone());
                                nav.push(Route::SplitUI);
                            },
                            "Submit"
                        }
                    }
                }
            }
            Err(error) => rsx! {
                div { class: "panel-heading has-text-centered", "{error.to_string()}" }
            },
        }
    } else {
        rsx! {
            div { class: "panel-heading has-text-centered",
                "Provide a total amount and at least two people to start splitting!"
            }
        }
    }
}

fn retrieve_people() -> Vec<Vec<String>> {
    let init_people_list: Vec<Vec<String>> = vec![]; // temp;
    use_persistent("people_list", || init_people_list).get()
}

fn set_people(new_people_list: Vec<String>) {
    let mut cached_people_list = retrieve_people();
    let empty: Vec<Vec<String>> = vec![];
    if cached_people_list.len() > 1 {
        cached_people_list.remove(0);
    }
    // Very inefficient, but shouldn't make a difference for such a small cache:
    let mut is_present = false;
    for people_list in cached_people_list.iter() {
        if *people_list == new_people_list {
            is_present = true;
        }
    }
    if !is_present {
        cached_people_list.push(new_people_list.clone());
        use_persistent("people_list", || empty).set(cached_people_list)
    };
}

#[component]
fn RetrieveCache(people_input: Signal<Vec<String>>) -> Element {
    let cache_people_list = retrieve_people();
    rsx! {
        if cache_people_list.len() > 0 {
            hr {}
            div { class: "panel-heading", "Or pick from recently used groups:" }
            for (idx , people) in cache_people_list.clone().into_iter().rev().enumerate() {
                if idx < 3 {
                    div { class: "columns",
                        div { class: "column",
                            div { class: "buttons",
                                for person in people.iter() {
                                    div {
                                        button {
                                            class: "button is-link",
                                            disabled: "true",
                                            "{person}"
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "column is-2 is-narrow",
                            button {
                                class: "button is-primary",
                                onclick: move |_| {
                                    people_input.set(people.clone());
                                },
                                "Select this group"
                            }
                        }
                        hr {}
                    }
                }
            }
        }
    }
}
