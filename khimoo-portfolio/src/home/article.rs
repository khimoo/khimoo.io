use yew::prelude::*;
use pulldown_cmark::{html, Parser};
use yew::virtual_dom::AttrValue;

#[function_component(ArticleIndex)]
pub fn article_index() -> Html {
    html! {
        <div style="padding: 16px;">
            <h1>{"Articles"}</h1>
            <ul>
                <li><a href="article/hello">{"hello"}</a></li>
            </ul>
        </div>
    }
}

fn sample_markdown(slug: &str) -> String {
    match slug {
        "hello" => include_str!("../../articles/hello.md").to_string(),
        _ => "# 記事が見つかりません".to_string(),
    }
}

#[derive(Properties, PartialEq)]
pub struct ArticleViewProps {
    pub slug: String,
}

#[function_component(ArticleView)]
pub fn article_view(props: &ArticleViewProps) -> Html {
    let markdown = sample_markdown(&props.slug);
    let parser = Parser::new(&markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let rendered = Html::from_html_unchecked(AttrValue::from(html_output));

    html! {
        <div style="padding: 16px;">
            <div class="markdown-body">{ rendered }</div>
        </div>
    }
}
