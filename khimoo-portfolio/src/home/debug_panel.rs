use yew::prelude::*;
use crate::home::types::{ForceSettings, NodeRegistry};

#[cfg(debug_assertions)]
pub mod debug_panel {
    use super::*;

    #[derive(Properties, PartialEq)]
    pub struct DebugPanelProps {
        pub force_settings: ForceSettings,
        pub node_registry: NodeRegistry,
        pub on_force_change: Callback<ForceSettings>,
    }

    #[function_component(DebugPanel)]
    pub fn debug_panel(props: &DebugPanelProps) -> Html {
        let force_settings = props.force_settings.clone();
        let node_registry = props.node_registry.clone();

        let on_repulsion_change = {
            let on_force_change = props.on_force_change.clone();
            let force_settings = force_settings.clone();
            Callback::from(move |e: Event| {
                let value = e.target_unchecked_into::<web_sys::HtmlInputElement>()
                    .value().parse().unwrap_or(68000.0);
                let mut new_settings = force_settings.clone();
                new_settings.repulsion_strength = value;
                on_force_change.emit(new_settings);
            })
        };

        let on_link_strength_change = {
            let on_force_change = props.on_force_change.clone();
            let force_settings = force_settings.clone();
            Callback::from(move |e: Event| {
                let value = e.target_unchecked_into::<web_sys::HtmlInputElement>()
                    .value().parse().unwrap_or(5000.0);
                let mut new_settings = force_settings.clone();
                new_settings.link_strength = value;
                on_force_change.emit(new_settings);
            })
        };

        let on_center_strength_change = {
            let on_force_change = props.on_force_change.clone();
            let force_settings = force_settings.clone();
            Callback::from(move |e: Event| {
                let value = e.target_unchecked_into::<web_sys::HtmlInputElement>()
                    .value().parse().unwrap_or(10000.0);
                let mut new_settings = force_settings.clone();
                new_settings.center_strength = value;
                on_force_change.emit(new_settings);
            })
        };

        let on_debug_mode_toggle = {
            let on_force_change = props.on_force_change.clone();
            let force_settings = force_settings.clone();
            Callback::from(move |e: Event| {
                let checked = e.target_unchecked_into::<web_sys::HtmlInputElement>()
                    .checked();
                let mut new_settings = force_settings.clone();
                new_settings.debug_mode = checked;
                on_force_change.emit(new_settings);
            })
        };

        let on_show_lines_toggle = {
            let on_force_change = props.on_force_change.clone();
            let force_settings = force_settings.clone();
            Callback::from(move |e: Event| {
                let checked = e.target_unchecked_into::<web_sys::HtmlInputElement>()
                    .checked();
                let mut new_settings = force_settings.clone();
                new_settings.show_connection_lines = checked;
                on_force_change.emit(new_settings);
            })
        };

        html! {
            <div class="debug-panel" style="
                position: fixed;
                top: 10px;
                right: 10px;
                background: rgba(0, 0, 0, 0.8);
                color: white;
                padding: 15px;
                border-radius: 8px;
                font-family: monospace;
                font-size: 12px;
                z-index: 1000;
                min-width: 250px;
                max-height: 80vh;
                overflow-y: auto;
            ">
                <h3 style="margin: 0 0 10px 0; color: #4CAF50;">{"Physics Debug Panel"}</h3>
                
                <div style="margin-bottom: 15px;">
                    <label style="display: block; margin-bottom: 5px;">
                        {"Repulsion Strength: "}
                        <span style="color: #FFC107;">{format!("{:.0}", force_settings.repulsion_strength)}</span>
                    </label>
                    <input 
                        type="range" 
                        min="0" 
                        max="100000" 
                        step="1000"
                        value={force_settings.repulsion_strength.to_string()}
                        onchange={on_repulsion_change}
                        style="width: 100%;"
                    />
                </div>

                <div style="margin-bottom: 15px;">
                    <label style="display: block; margin-bottom: 5px;">
                        {"Link Strength: "}
                        <span style="color: #FFC107;">{format!("{:.0}", force_settings.link_strength)}</span>
                    </label>
                    <input 
                        type="range" 
                        min="0" 
                        max="20000" 
                        step="500"
                        value={force_settings.link_strength.to_string()}
                        onchange={on_link_strength_change}
                        style="width: 100%;"
                    />
                </div>

                <div style="margin-bottom: 15px;">
                    <label style="display: block; margin-bottom: 5px;">
                        {"Center Strength: "}
                        <span style="color: #FFC107;">{format!("{:.0}", force_settings.center_strength)}</span>
                    </label>
                    <input 
                        type="range" 
                        min="0" 
                        max="50000" 
                        step="1000"
                        value={force_settings.center_strength.to_string()}
                        onchange={on_center_strength_change}
                        style="width: 100%;"
                    />
                </div>

                <div style="margin-bottom: 15px;">
                    <label style="display: flex; align-items: center; gap: 8px;">
                        <input 
                            type="checkbox" 
                            checked={force_settings.debug_mode}
                            onchange={on_debug_mode_toggle}
                        />
                        {"Debug Mode"}
                    </label>
                </div>

                <div style="margin-bottom: 15px;">
                    <label style="display: flex; align-items: center; gap: 8px;">
                        <input 
                            type="checkbox" 
                            checked={force_settings.show_connection_lines}
                            onchange={on_show_lines_toggle}
                        />
                        {"Show Connection Lines"}
                    </label>
                </div>

                <div style="border-top: 1px solid #333; padding-top: 10px;">
                    <p style="margin: 5px 0; color: #81C784;">
                        {format!("Total Nodes: {}", node_registry.positions.len())}
                    </p>
                    <p style="margin: 5px 0; color: #81C784;">
                        {format!("Connection Lines: {}", node_registry.connection_lines.len())}
                    </p>
                    <p style="margin: 5px 0; color: #81C784;">
                        {format!("Categories: {}", node_registry.get_all_categories().len())}
                    </p>
                    <p style="margin: 5px 0; color: #81C784;">
                        {format!("Author Node: {}", if node_registry.get_author_node_id().is_some() { "Yes" } else { "No" })}
                    </p>
                </div>

                <div style="border-top: 1px solid #333; padding-top: 10px; margin-top: 10px;">
                    <h4 style="margin: 0 0 5px 0; color: #FF9800;">{"Categories:"}</h4>
                    {for node_registry.get_all_categories().iter().map(|category| {
                        let _color = node_registry.get_category_color(category);
                        let count = node_registry.get_nodes_by_category(category).len();
                        html! {
                            <div style="margin: 2px 0; font-size: 11px;">
                                <span style="color: {_color.primary.clone()}; font-weight: bold;">{category}</span>
                                <span style="color: #999;">{" ("}{count}{")"}</span>
                            </div>
                        }
                    })}
                </div>
            </div>
        }
    }
}

#[cfg(not(debug_assertions))]
pub mod debug_panel {
    use super::*;

    #[derive(Properties, PartialEq)]
    pub struct DebugPanelProps {
        pub force_settings: ForceSettings,
        pub node_registry: NodeRegistry,
        pub on_force_change: Callback<ForceSettings>,
    }

    #[function_component(DebugPanel)]
    pub fn debug_panel(_props: &DebugPanelProps) -> Html {
        html! {} // リリースビルドでは何も表示しない
    }
}
