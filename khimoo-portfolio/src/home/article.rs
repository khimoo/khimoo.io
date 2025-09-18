use yew::prelude::*;
use pulldown_cmark::{html, Parser};
use yew::virtual_dom::AttrValue;
use super::data_loader::{use_article_content, use_lightweight_articles};
use yew_router::prelude::*;

#[function_component(ArticleIndex)]
pub fn article_index() -> Html {
    let (articles, loading, error) = use_lightweight_articles();
    
    if *loading {
        return html! {
            <div style="padding: 16px;">
                <h1>{"Articles"}</h1>
                <p>{"Loading articles..."}</p>
            </div>
        };
    }
    
    if let Some(err) = error.as_ref() {
        return html! {
            <div style="padding: 16px;">
                <h1>{"Articles"}</h1>
                <p style="color: red;">{format!("Error loading articles: {}", err)}</p>
            </div>
        };
    }
    
    html! {
        <div style="padding: 16px;">
            <h1>{"Articles"}</h1>
            <div style="margin-bottom: 20px;">
                <Link<Route> to={Route::Home}>
                    <button style="padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;">
                        {"← Back to Home"}
                    </button>
                </Link<Route>>
            </div>
            {
                if let Some(articles_list) = articles.as_ref() {
                    html! {
                        <ul style="list-style: none; padding: 0;">
                            {
                                articles_list.iter().map(|article| {
                                    html! {
                                        <li key={article.slug.clone()} style="margin-bottom: 20px; padding: 16px; border: 1px solid #ddd; border-radius: 8px;">
                                            <h3 style="margin: 0 0 8px 0;">
                                                <Link<Route> to={Route::ArticleShow { slug: article.slug.clone() }}>
                                                    {&article.title}
                                                </Link<Route>>
                                            </h3>
                                            {
                                                if let Some(summary) = &article.summary {
                                                    html! { <p style="color: #666; margin: 8px 0;">{summary}</p> }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            <div style="font-size: 12px; color: #999;">
                                                {
                                                    if let Some(category) = &article.metadata.category {
                                                        html! { <span style="margin-right: 16px;">{"Category: "}{category}</span> }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                <span>{"Links: "}{article.inbound_count}</span>
                                            </div>
                                        </li>
                                    }
                                }).collect::<Html>()
                            }
                        </ul>
                    }
                } else {
                    html! { <p>{"No articles found."}</p> }
                }
            }
        </div>
    }
}

// Import the Route enum
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

#[derive(Properties, PartialEq)]
pub struct ArticleViewProps {
    pub slug: String,
}

#[function_component(ArticleView)]
pub fn article_view(props: &ArticleViewProps) -> Html {
    let (article, loading, error) = use_article_content(Some(props.slug.clone()));
    
    if *loading {
        return html! {
            <div style="padding: 16px;">
                <div style="margin-bottom: 20px;">
                    <Link<Route> to={Route::Home}>
                        <button style="padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;">
                            {"← Back to Home"}
                        </button>
                    </Link<Route>>
                </div>
                <h1>{"Loading article..."}</h1>
                <div style="margin-top: 20px;">
                    <div style="border: 4px solid #f3f3f3; border-top: 4px solid #3498db; border-radius: 50%; width: 40px; height: 40px; animation: spin 2s linear infinite;"></div>
                </div>
            </div>
        };
    }
    
    if let Some(err) = error.as_ref() {
        return html! {
            <div style="padding: 16px;">
                <div style="margin-bottom: 20px;">
                    <Link<Route> to={Route::Home}>
                        <button style="padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;">
                            {"← Back to Home"}
                        </button>
                    </Link<Route>>
                </div>
                <h1>{"Article Not Found"}</h1>
                <p style="color: red;">{format!("Error: {}", err)}</p>
                <p>{"The article you're looking for doesn't exist or couldn't be loaded."}</p>
            </div>
        };
    }
    
    if let Some(article_data) = article.as_ref() {
        // Convert markdown to HTML
        let parser = Parser::new(&article_data.content);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        let rendered = Html::from_html_unchecked(AttrValue::from(html_output));
        
        html! {
            <>
                <style>
                    {"@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }"}
                    {".markdown-body { line-height: 1.6; } .markdown-body h1, .markdown-body h2, .markdown-body h3 { margin-top: 24px; margin-bottom: 16px; } .markdown-body p { margin-bottom: 16px; } .markdown-body ul, .markdown-body ol { margin-bottom: 16px; padding-left: 30px; } .markdown-body code { background: #f6f8fa; padding: 2px 4px; border-radius: 3px; font-size: 85%; } .markdown-body pre { background: #f6f8fa; padding: 16px; border-radius: 6px; overflow: auto; } .markdown-body blockquote { border-left: 4px solid #dfe2e5; padding-left: 16px; color: #6a737d; margin: 0 0 16px 0; }"}
                </style>
                <div style="padding: 16px; max-width: 800px; margin: 0 auto;">
                    <div style="margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center;">
                        <Link<Route> to={Route::Home}>
                            <button style="padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;">
                                {"← Back to Home"}
                            </button>
                        </Link<Route>>
                        <Link<Route> to={Route::ArticleIndex}>
                            <button style="padding: 8px 16px; background: #6c757d; color: white; border: none; border-radius: 4px; cursor: pointer;">
                                {"All Articles"}
                            </button>
                        </Link<Route>>
                    </div>
                    
                    <article>
                        <header style="margin-bottom: 32px; padding-bottom: 16px; border-bottom: 1px solid #eee;">
                            <h1 style="margin: 0 0 16px 0; font-size: 2.5em; color: #333;">{&article_data.title}</h1>
                            <div style="font-size: 14px; color: #666; display: flex; gap: 16px; flex-wrap: wrap;">
                                {
                                    if let Some(category) = &article_data.metadata.category {
                                        html! { <span>{"Category: "}<strong>{category}</strong></span> }
                                    } else {
                                        html! {}
                                    }
                                }
                                {
                                    if let Some(importance) = article_data.metadata.importance {
                                        html! { <span>{"Importance: "}<strong>{importance}{"/5"}</strong></span> }
                                    } else {
                                        html! {}
                                    }
                                }
                                <span>{"Inbound links: "}<strong>{article_data.inbound_count}</strong></span>
                                {
                                    if !article_data.metadata.tags.is_empty() {
                                        html! {
                                            <span>
                                                {"Tags: "}
                                                {
                                                    article_data.metadata.tags.iter().enumerate().map(|(i, tag)| {
                                                        html! {
                                                            <>
                                                                {if i > 0 { ", " } else { "" }}
                                                                <span style="background: #e9ecef; padding: 2px 6px; border-radius: 3px; font-size: 12px;">{tag}</span>
                                                            </>
                                                        }
                                                    }).collect::<Html>()
                                                }
                                            </span>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </header>
                        
                        <div class="markdown-body">
                            { rendered }
                        </div>
                        
                        {
                            if !article_data.outbound_links.is_empty() {
                                html! {
                                    <footer style="margin-top: 48px; padding-top: 24px; border-top: 1px solid #eee;">
                                        <h3>{"Related Articles"}</h3>
                                        <ul style="list-style: none; padding: 0;">
                                            {
                                                article_data.outbound_links.iter().map(|link| {
                                                    html! {
                                                        <li key={link.target_slug.clone()} style="margin-bottom: 8px;">
                                                            <Link<Route> to={Route::ArticleShow { slug: link.target_slug.clone() }}>
                                                                {&link.target_slug}
                                                            </Link<Route>>
                                                            {
                                                                if !link.context.is_empty() {
                                                                    html! { <span style="color: #666; font-size: 12px; margin-left: 8px;">{format!("\"{}\"", &link.context)}</span> }
                                                                } else {
                                                                    html! {}
                                                                }
                                                            }
                                                        </li>
                                                    }
                                                }).collect::<Html>()
                                            }
                                        </ul>
                                    </footer>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </article>
                </div>
            </>
        }
    } else {
        html! {
            <div style="padding: 16px;">
                <div style="margin-bottom: 20px;">
                    <Link<Route> to={Route::Home}>
                        <button style="padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;">
                            {"← Back to Home"}
                        </button>
                    </Link<Route>>
                </div>
                <h1>{"Article Not Found"}</h1>
                <p>{"The article you're looking for doesn't exist."}</p>
            </div>
        }
    }
}
