use super::physics_sim::{PhysicsWorld, Viewport};
use super::types::*;
use super::data_loader::{use_articles_data, ArticlesData, ProcessedArticle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_hooks::{use_effect_update_with_deps, use_interval, use_window_scroll, UseMeasureState};
use yew_router::prelude::*;

// Import the Route enum from main.rs
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

// 作者ノード検索機能：author_imageフィールドを持つ記事を検索
fn find_author_article(articles_data: &ArticlesData) -> Option<&ProcessedArticle> {
    // author_imageフィールドを持つ記事をすべて収集
    let author_articles: Vec<&ProcessedArticle> = articles_data.articles
        .iter()
        .filter(|article| article.metadata.author_image.is_some())
        .collect();
    
    // ログ出力（WebAssembly環境でのみ実行）
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("Found {} articles with author_image field", author_articles.len()).into());
    }
    
    match author_articles.len() {
        0 => {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&"No author_image found in any articles".into());
            None
        }
        1 => {
            let author_article = author_articles[0];
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!(
                "Found author article: '{}' with image: '{}'", 
                author_article.title,
                author_article.metadata.author_image.as_ref().unwrap()
            ).into());
            Some(author_article)
        }
        _ => {
            // 複数の作者記事が存在する場合は警告を出力し、最初のものを使用
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::warn_1(&format!(
                    "Multiple articles with author_image found ({}). Using the first one: '{}'",
                    author_articles.len(),
                    author_articles[0].title
                ).into());
                
                // すべての作者記事をログに出力
                for (index, article) in author_articles.iter().enumerate() {
                    web_sys::console::log_1(&format!(
                        "  {}: '{}' ({})", 
                        index + 1,
                        article.title,
                        article.metadata.author_image.as_ref().unwrap()
                    ).into());
                }
            }
            
            Some(author_articles[0])
        }
    }
}

// ArticlesDataからNodeRegistryを生成する関数
fn create_node_registry_from_articles(articles_data: &ArticlesData, container_bound: &ContainerBound) -> (NodeRegistry, HashMap<NodeId, String>) {
    let mut reg = NodeRegistry::new();
    let mut slug_to_id = HashMap::new();
    let mut id_to_slug = HashMap::new();
    let mut next_id = 1u32;
    
    // コンテナの中心を計算
    let center_x = container_bound.width / 2.0;
    let center_y = container_bound.height / 2.0;
    
    // デバッグ情報
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("Container bound in create_node_registry: {:?}", container_bound).into());
        web_sys::console::log_1(&format!("Calculated center: ({}, {})", center_x, center_y).into());
    }
    
    // 作者ノードの検索と作成
    if let Some(author_article) = find_author_article(articles_data) {
        // メタデータベースの作者ノード作成
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!(
            "Creating metadata-based author node: '{}' with image: '{}'", 
            author_article.title,
            author_article.metadata.author_image.as_ref().unwrap()
        ).into());
        
        let author_content = NodeContent::Author {
            name: author_article.title.clone(),
            image_url: author_article.metadata.author_image.clone().unwrap(),
            bio: None, // 将来的に author_bio フィールドを追加可能
        };
        
        reg.add_node(
            NodeId(0),
            Position { x: center_x, y: center_y },
            60, // 作者ノードは大きめ
            author_content,
        );
        
        slug_to_id.insert(author_article.slug.clone(), NodeId(0));
        id_to_slug.insert(NodeId(0), author_article.slug.clone());
    } else {
        // フォールバック：従来のハードコーディング方式
        #[cfg(target_arch = "wasm32")]
        web_sys::console::warn_1(&"No author_image found, using fallback author node".into());
        
        reg.add_node(
            NodeId(0),
            Position { x: center_x, y: center_y },
            40,
            NodeContent::Text("Author".to_string()),
        );
        
        slug_to_id.insert("author".to_string(), NodeId(0));
        id_to_slug.insert(NodeId(0), "author".to_string());
    }
    
    reg.set_node_importance(NodeId(0), 5); // 最高重要度
    reg.set_node_inbound_count(NodeId(0), 0);
    
    // home_display=trueの記事のみをノードとして追加
    let home_articles: Vec<_> = articles_data.articles.iter()
        .filter(|article| article.metadata.home_display)
        .collect();
    
    // デバッグ情報をコンソールに出力
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("Total articles: {}", articles_data.articles.len()).into());
        web_sys::console::log_1(&format!("Home articles count: {}", home_articles.len()).into());
        for article in &home_articles {
            web_sys::console::log_1(&format!("Home article: {} ({})", article.title, article.slug).into());
        }
    }
    
    // home_articlesが空の場合は作者ノードのみ返す
    if home_articles.is_empty() {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::warn_1(&"No home articles found!".into());
        return (reg, id_to_slug);
    }
    
    // 円形にノードを配置するための計算（コンテナサイズに基づく）
    let radius = (container_bound.width.min(container_bound.height) * 0.3).max(150.0); // コンテナサイズの30%、最小150px
    let angle_step = 2.0 * std::f32::consts::PI / home_articles.len() as f32;
    
    for (index, article) in home_articles.iter().enumerate() {
        let angle = index as f32 * angle_step;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        
        let node_id = NodeId(next_id);
        reg.add_node(
            node_id,
            Position { x, y },
            30, // ベースサイズ
            NodeContent::Text(article.title.clone()),
        );
        
        // 重要度とリンク数を設定
        reg.set_node_importance(node_id, article.metadata.importance.unwrap_or(3));
        reg.set_node_inbound_count(node_id, article.inbound_count);
        
        slug_to_id.insert(article.slug.clone(), node_id);
        id_to_slug.insert(node_id, article.slug.clone());
        next_id += 1;
    }
    
    // 作者ノードから全記事への接続を追加
    for (_, &article_id) in &slug_to_id {
        if article_id.0 != 0 { // 作者ノード以外
            reg.add_edge(NodeId(0), article_id);
        }
    }
    
    // 記事間のリンクを追加
    for article in &home_articles {
        if let Some(&from_id) = slug_to_id.get(&article.slug) {
            for link in &article.outbound_links {
                if let Some(&to_id) = slug_to_id.get(&link.target_slug) {
                    reg.add_edge(from_id, to_id);
                }
            }
        }
    }
    
    (reg, id_to_slug)
}

#[derive(Properties, PartialEq)]
pub struct NodeGraphContainerProps {
    pub container_ref: NodeRef,
    pub container_measure: UseMeasureState,
    pub container_bound: ContainerBound,
}

#[function_component(NodeGraphContainer)]
pub fn node_graph_container(props: &NodeGraphContainerProps) -> Html {
    let dragged_node_id = use_state(|| None::<NodeId>);
    let viewport = use_state(Viewport::default);
    let force_settings = use_state(ForceSettings::default);

    // データローダーを使用して記事データを取得
    let (articles_data, loading, error) = use_articles_data();
        
    // 記事データが読み込まれたらノードレジストリと物理世界を一度だけ初期化
    let node_registry = use_state(|| Rc::new(RefCell::new(NodeRegistry::new())));
    let node_slug_mapping = use_state(|| HashMap::<NodeId, String>::new());
    let physics_world = use_state(|| {
        let empty_registry = Rc::new(RefCell::new(NodeRegistry::new()));
        // 初期化時はデフォルトのContainerBoundを使用（後で更新される）
        let default_bound = ContainerBound::default();
        Rc::new(RefCell::new(PhysicsWorld::new(
            empty_registry,
            &viewport,
            *force_settings,
            default_bound,
        )))
    });

    // 記事データが初回読み込まれた時のみ初期化（静的データなので一度だけ）
    let initialized = use_state(|| false);
    if let Some(data) = articles_data.as_ref() {
        if !*initialized {
            web_sys::console::log_1(&format!("Initializing with container_bound: {:?}", props.container_bound).into());
            
            let (new_registry, slug_mapping) = create_node_registry_from_articles(data, &props.container_bound);
            let registry_rc = Rc::new(RefCell::new(new_registry));
            node_registry.set(Rc::clone(&registry_rc));
            node_slug_mapping.set(slug_mapping);
            
            let new_physics_world = PhysicsWorld::new(
                registry_rc,
                &viewport,
                *force_settings,
                props.container_bound.clone(),
            );
            physics_world.set(Rc::new(RefCell::new(new_physics_world)));
            initialized.set(true);
        }
    }

    // 力の設定が変更されたらPhysicsWorldを更新
    {
        let physics_world = physics_world.clone();
        let force_settings_clone = force_settings.clone();
        use_effect_update_with_deps(
            move |_| {
                physics_world.borrow_mut().update_force_settings(*force_settings_clone);
                || {}
            },
            force_settings.clone(),
        );
    }

    // コンテナ境界が変更されたらPhysicsWorldを更新
    {
        let physics_world = physics_world.clone();
        use_effect_update_with_deps(
            move |container_bound| {
                web_sys::console::log_1(&format!("Container bound changed in effect: {:?}", container_bound).into());
                physics_world.borrow_mut().update_container_bound(container_bound.clone());
                || {}
            },
            props.container_bound.clone(),
        );
    }

    let scroll = use_window_scroll();

    // ドラッグ開始位置を追跡
    let drag_start_pos = use_state(|| None::<(i32, i32)>);
    let is_dragging = use_state(|| false);

    let on_mouse_move = {
        let dragged_node_id = dragged_node_id.clone();
        let physics_world = physics_world.clone();
        let viewport = viewport.clone();
        let drag_start_pos = drag_start_pos.clone();
        let is_dragging = is_dragging.clone();
        Callback::from(move |e: MouseEvent| {
            if let Some(id) = *dragged_node_id {
                // ドラッグ距離をチェック
                if let Some((start_x, start_y)) = *drag_start_pos {
                    let dx = e.client_x() - start_x;
                    let dy = e.client_y() - start_y;
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    
                    // 5px以上移動したらドラッグ開始
                    if distance > 5.0 && !*is_dragging {
                        is_dragging.set(true);
                        physics_world.borrow_mut().set_node_kinematic(id);
                    }
                    
                    // ドラッグ中の場合のみノード位置を更新
                    if *is_dragging {
                        let mut world = physics_world.borrow_mut();
                        let screen_pos = Position {
                            x: (e.client_x() + scroll.0 as i32) as f32,
                            y: (e.client_y() + scroll.1 as i32) as f32,
                        };
                        world.set_node_position(id, &screen_pos, &viewport);
                    }
                }
            }
        })
    };

    // ノードクリック時のナビゲーション処理
    let navigator = use_navigator().unwrap();
    let on_node_click = {
        let navigator = navigator.clone();
        let node_slug_mapping = node_slug_mapping.clone();
        Callback::from(move |node_id: NodeId| {
            if let Some(slug) = node_slug_mapping.get(&node_id) {
                // 作者ノードの場合はホームに留まる
                if slug == "author" {
                    web_sys::console::log_1(&"Author node clicked - staying on home page".into());
                    return;
                }
                
                // 記事ページに遷移
                web_sys::console::log_1(&format!("Navigating to article: {}", slug).into());
                let route = Route::ArticleShow { slug: slug.clone() };
                navigator.push(&route);
            }
        })
    };
    
    let on_mouse_down = {
        let dragged_node_id = dragged_node_id.clone();
        let drag_start_pos = drag_start_pos.clone();
        let is_dragging = is_dragging.clone();
        Callback::from(move |(id, e): (NodeId, MouseEvent)| {
            // ドラッグ開始位置を記録
            drag_start_pos.set(Some((e.client_x(), e.client_y())));
            is_dragging.set(false);
            dragged_node_id.set(Some(id));
        })
    };

    let on_mouse_up = {
        let dragged_node_id = dragged_node_id.clone();
        let physics_world = physics_world.clone();
        let drag_start_pos = drag_start_pos.clone();
        let is_dragging = is_dragging.clone();
        let on_node_click = on_node_click.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(id) = *dragged_node_id {
                // ドラッグしていた場合は物理状態をリセット
                if *is_dragging {
                    physics_world.borrow_mut().set_node_dynamic(id);
                } else {
                    // ドラッグしていない場合はクリックイベントを発火
                    on_node_click.emit(id);
                }
            }
            
            // 状態をリセット
            dragged_node_id.set(None);
            drag_start_pos.set(None);
            is_dragging.set(false);
        })
    };

    let rerender = use_state(|| ());

    {
        let physics_world = physics_world.clone();
        let viewport = viewport.clone();
        let rerender = rerender.clone();
        use_interval(
            move || {
                let mut world = physics_world.borrow_mut();
                world.step(&viewport);
                rerender.set(());
            },
            8, // ~120fps
        );
    }

    // 力の設定を更新するコールバック
    let on_repulsion_strength_change = {
        let force_settings = force_settings.clone();
        Callback::from(move |e: Event| {
            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
            let value = target.value().parse::<f32>().unwrap_or(50000.0);
            let mut settings = *force_settings;
            settings.repulsion_strength = value;
            force_settings.set(settings);
        })
    };

    let on_repulsion_distance_change = {
        let force_settings = force_settings.clone();
        Callback::from(move |e: Event| {
            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
            let value = target.value().parse::<f32>().unwrap_or(20.0);
            let mut settings = *force_settings;
            settings.repulsion_min_distance = value;
            force_settings.set(settings);
        })
    };


    let on_link_strength_change = {
        let force_settings = force_settings.clone();
        Callback::from(move |e: Event| {
            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
            let value = target.value().parse::<f32>().unwrap_or(5000.0);
            let mut settings = *force_settings;
            settings.link_strength = value;
            force_settings.set(settings);
        })
    };

    let on_center_strength_change = {
        let force_settings = force_settings.clone();
        Callback::from(move |e: Event| {
            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
            let value = target.value().parse::<f32>().unwrap_or(50.0);
            let mut settings = *force_settings;
            settings.center_strength = value;
            force_settings.set(settings);
        })
    };

    let on_center_damping_change = {
        let force_settings = force_settings.clone();
        Callback::from(move |e: Event| {
            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
            let value = target.value().parse::<f32>().unwrap_or(5.0);
            let mut settings = *force_settings;
            settings.center_damping = value;
            force_settings.set(settings);
        })
    };

    // ローディング中やエラー時の表示
    if *loading {
        return html! {
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh; background: #f0f0f0;">
                <div style="text-align: center;">
                    <h2>{"記事データを読み込み中..."}</h2>
                    <div style="margin-top: 20px;">
                        <div style="border: 4px solid #f3f3f3; border-top: 4px solid #3498db; border-radius: 50%; width: 40px; height: 40px; animation: spin 2s linear infinite; margin: 0 auto;"></div>
                    </div>
                </div>
            </div>
        };
    }

    if let Some(err) = error.as_ref() {
        return html! {
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh; background: #f0f0f0;">
                <div style="text-align: center; color: #e74c3c;">
                    <h2>{"データの読み込みに失敗しました"}</h2>
                    <p>{format!("エラー: {}", err)}</p>
                </div>
            </div>
        };
    }

    html! {
        <>
            <style>
                {"@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }"}
            </style>
            <div
                style="position: static; width: 100vw; height: 100vh; background: #f0f0f0;"
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up}
                ref={props.container_ref.clone()}
            >
                <h1>{"Interactive Mindmap Portfolio"}</h1>
                <p>{ format!("記事数: {}", node_registry.borrow().positions.len()) }</p>
                <p>{ format!("{:?}", props.container_bound)}</p>
                {{
                    // 力の設定UI
                    html! {
                        <div style="position: absolute; top: 20px; right: 20px; background: rgba(0,0,0,0.8); color: white; padding: 20px; border-radius: 10px; z-index: 100;">
                            <h3 style="margin: 0 0 15px 0;">{"力の設定"}</h3>
                            <div style="margin-bottom: 15px;">
                                <label>{"反発力の強さ: "}{force_settings.repulsion_strength as i32}</label><br/>
                                <input
                                    type="range"
                                    min="0"
                                    max="200000"
                                    step="1000"
                                    value={force_settings.repulsion_strength.to_string()}
                                    onchange={on_repulsion_strength_change.clone()}
                                    style="width: 200px;"
                                />
                            </div>
                            <div style="margin-bottom: 15px;">
                                <label>{"反発力の最小距離: "}{force_settings.repulsion_min_distance as i32}</label><br/>
                                <input
                                    type="range"
                                    min="0"
                                    max="1000"
                                    step="5"
                                    value={force_settings.repulsion_min_distance.to_string()}
                                    onchange={on_repulsion_distance_change.clone()}
                                    style="width: 200px;"
                                />
                            </div>
                            <div style="margin-bottom: 15px;">
                                <label>{"中心力の強さ: "}{force_settings.center_strength as i32}</label><br/>
                                <input
                                    type="range"
                                    min="0"
                                    max="3000"
                                    step="1"
                                    value={force_settings.center_strength.to_string()}
                                    onchange={on_center_strength_change.clone()}
                                    style="width: 200px;"
                                />
                            </div>
                            <div style="margin-bottom: 15px;">
                                <label>{"中心減衰: "}{force_settings.center_damping as i32}</label><br/>
                                <input
                                    type="range"
                                    min="0"
                                    max="50"
                                    step="1"
                                    value={force_settings.center_damping.to_string()}
                                    onchange={on_center_damping_change.clone()}
                                    style="width: 200px;"
                                />
                            </div>
                            <div style="margin-bottom: 15px;">
                                <label>{"リンク力の強さ: "}{force_settings.link_strength as i32}</label><br/>
                                <input
                                    type="range"
                                    min="0"
                                    max="50000"
                                    step="100"
                                    value={force_settings.link_strength.to_string()}
                                    onchange={on_link_strength_change.clone()}
                                    style="width: 200px;"
                                />
                            </div>
                        </div>
                    }
                }}
                {{
                    // 背景のエッジ描画
                    let reg = node_registry.borrow();
                    html!{
                        <svg style="position: absolute; left: 0; top: 0; width: 100vw; height: 100vh; z-index: 1; pointer-events: none;">
                            {
                                reg.iter_edges().filter_map(|(a, b)| {
                                    let p1 = reg.positions.get(a)?;
                                    let p2 = reg.positions.get(b)?;
                                    Some(html!{
                                        <line
                                            x1={format!("{:.2}", p1.x)}
                                            y1={format!("{:.2}", p1.y)}
                                            x2={format!("{:.2}", p2.x)}
                                            y2={format!("{:.2}", p2.y)}
                                            stroke="#8a8a8a"
                                            stroke-width="1.5"
                                        />
                                    })
                                }).collect::<Html>()
                            }
                        </svg>
                    }
                }}
                {
                    node_registry.borrow().iter().map(|(id, pos, radius, content)| {
                        let registry = node_registry.borrow();
                        let importance = registry.get_node_importance(*id);
                        let inbound_count = registry.get_node_inbound_count(*id);
                        drop(registry);
                        
                        let on_mouse_down = {
                            let on_mouse_down = on_mouse_down.clone();
                            let id = *id;
                            Callback::from(move |e: MouseEvent| {
                                e.stop_propagation();
                                on_mouse_down.emit((id, e));
                            })
                        };
                        
                        html!{
                            <NodeComponent
                                key={id.0}
                                id={*id}
                                pos={*pos}
                                radius={*radius}
                                content={content.clone()}
                                {importance}
                                {inbound_count}
                                {on_mouse_down}
                            />
                        }
                    }).collect::<Html>()
                }
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct NodeProps {
    pub id: NodeId,
    pub pos: Position,
    pub radius: i32,
    pub content: NodeContent,
    pub on_mouse_down: Callback<MouseEvent>,
    pub importance: Option<u8>,
    pub inbound_count: usize,
}

#[function_component(NodeComponent)]
fn node_component(props: &NodeProps) -> Html {
    // 重要度とリンク数に基づいて動的にサイズを計算
    let dynamic_radius = calculate_dynamic_radius(props.radius, props.importance, props.inbound_count);
    
    html! {
        <div
            key={props.id.0.to_string()}
            onmousedown={props.on_mouse_down.clone()}
            style={format!(
                "position: absolute;
                width: {}px;
                height: {}px;
                background-color: black;
                border-radius: 50%;
                transform: translate(-50%, -50%);
                left: {}px;
                top: {}px;
                box-shadow: 0 4px 8px rgba(0,0,0,0.2);
                z-index: 10;
                display: flex;
                justify-content: center;
                align-items: center;
                cursor: pointer;
                transition: transform 0.2s ease-in-out;
                user-select: none;",
                2 * dynamic_radius,
                2 * dynamic_radius,
                props.pos.x,
                props.pos.y
            )}
        >
            <div style="max-width: 80%; max-height: 80%; overflow: hidden; pointer-events: none;">
                {props.content.render_content()}
            </div>
        </div>
    }
}

// 重要度とリンク数に基づいて動的サイズを計算する関数
fn calculate_dynamic_radius(base_radius: i32, importance: Option<u8>, inbound_count: usize) -> i32 {
    let mut size = base_radius;
    
    // 重要度に基づくサイズ調整 (1-5スケール)
    if let Some(imp) = importance {
        let importance_bonus = match imp {
            1 => -5,  // 小さく
            2 => -2,  
            3 => 0,   // ベースサイズ
            4 => 5,   // 大きく
            5 => 10,  // 最大
            _ => 0,
        };
        size += importance_bonus;
    }
    
    // インバウンドリンク数に基づくサイズ調整
    let popularity_bonus = (inbound_count as f32).sqrt() as i32 * 3;
    size += popularity_bonus;
    
    // 最小・最大サイズの制限
    size.clamp(15, 60)
}