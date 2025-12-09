use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header style="background:#fff;padding:12px;border-bottom:1px solid #eee;">
            <div style="max-width:1000px;margin:0 auto;display:flex;align-items:center;justify-content:flex-start;gap:12px;">
                <a href="/" style="text-decoration:none;color:#333;font-weight:600;">{"Home"}</a>
            </div>
        </header>
    }
}
