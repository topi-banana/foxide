use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RoleSelectorProps {
    pub roles: Vec<(u64, String)>,
    pub selected: Option<u64>,
    pub on_select: Callback<u64>,
    pub disabled: bool,
}

pub struct RoleSelector;

pub enum RoleSelectorMsg {
    Change(String),
}

impl Component for RoleSelector {
    type Message = RoleSelectorMsg;
    type Properties = RoleSelectorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RoleSelectorMsg::Change(value) => {
                if let Ok(id) = value.parse::<u64>() {
                    ctx.props().on_select.emit(id);
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let selected = props.selected;
        let onchange = ctx.link().callback(|ev: Event| {
            let target: HtmlSelectElement = ev.target().unwrap().dyn_into().unwrap();
            RoleSelectorMsg::Change(target.value())
        });
        let disabled = props.disabled || props.roles.is_empty();
        let value = selected.map(|id| id.to_string()).unwrap_or_default();

        html! {
            <select class="select select-bordered w-full" {onchange} {disabled} {value}>
                <option value="" disabled=true selected={selected.is_none()}>
                    {"Select a role..."}
                </option>
                { for props.roles.iter().map(|(id, name)| {
                    let id_str = id.to_string();
                    let is_selected = selected == Some(*id);
                    html! { <option value={id_str} selected={is_selected}>{name}</option> }
                }) }
            </select>
        }
    }
}
