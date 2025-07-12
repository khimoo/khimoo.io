use khimoo_portfolio::App;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/main")]
    Home,
    #[at("/admin")]
    Admin,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<App/>},
        Route::Admin => html! { <h1> {"Admin"} </h1> },
    }
}

#[function_component(Root)]
fn root() -> Html {
    // 本番環境ではリポジトリ名をベースパスに設定
    let basename = if cfg!(debug_assertions) {
        "/".to_string()
    } else {
        "/khimoo-portfolio/".to_string() // 実際のリポジトリ名に変更
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
