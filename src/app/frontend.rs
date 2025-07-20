use crate::app::display::DisplaySplits;
use crate::app::splash::CreateReceiptSplash;
use crate::app::split::SplitUI;
use crate::core::receipt::Receipt;
use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/bulma.css");
pub static RECEIPT_STATE: GlobalSignal<Option<Receipt>> = Signal::global(|| None);

#[derive(Routable, Clone, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[route("/create")]
    #[redirect("/", || Route::CreateReceiptSplash {})]
    CreateReceiptSplash,
    #[route("/split")]
    SplitUI,
    #[route("/display")]
    DisplaySplits,
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        Router::<Route> {}
    }
}
