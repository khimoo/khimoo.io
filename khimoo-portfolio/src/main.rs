use yew::prelude::*;
use yew_router::prelude::*;

use khimoo_portfolio::home::app::Home;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/admin")]
    Admin,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<Home/>},
        Route::Admin => html! { <h1> {"Admin"} </h1> },
    }
}

#[function_component(Root)]
fn root() -> Html {
    let basename = if cfg!(debug_assertions) {
        "/".to_string()
    } else {
        "/khimoo.io/".to_string() // github pagesのURL
    };

    html! {
        <BrowserRouter basename={basename}>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Root>::new().render();
}
