use yew::prelude::*;
use yew_router::prelude::*;

use khimoo_portfolio::home::app::Home;
use khimoo_portfolio::home::article::{ArticleIndex, ArticleView};
use khimoo_portfolio::home::header::Header;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/admin")]
    Admin,
    #[at("/article")]
    ArticleIndex,
    #[at("/article/:slug")]
    ArticleShow { slug: String },
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<Home/>},
        Route::Admin => html! { <h1> {"Admin"} </h1> },
        Route::ArticleIndex => html! { <ArticleIndex /> },
        Route::ArticleShow { slug } => html! { <ArticleView slug={slug} /> },
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
            <>
                <Header />
                <Switch<Route> render={switch} />
            </>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Root>::new().render();
}
