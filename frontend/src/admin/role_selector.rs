use leptos::prelude::*;

/// Reusable Discord role selector dropdown.
///
/// - `roles`    – available roles as `(id, name)` pairs
/// - `selected` – currently selected role id (None = placeholder shown)
/// - `on_select` – called with the newly selected role id
/// - `disabled` – whether the control is disabled
#[component]
pub fn RoleSelector(
    roles: ReadSignal<Vec<(u64, String)>>,
    selected: Signal<Option<u64>>,
    on_select: impl Fn(u64) + 'static + Copy,
    disabled: Signal<bool>,
) -> AnyView {
    view! {
        <select
            class="select select-bordered w-full"
            prop:value=move || selected.get().map(|id| id.to_string()).unwrap_or_default()
            on:change=move |ev| {
                let value = event_target_value(&ev);
                if let Ok(id) = value.parse::<u64>() {
                    on_select(id);
                }
            }
            disabled=move || disabled.get() || roles.get().is_empty()
        >
            <option value="" disabled selected=move || selected.get().is_none()>
                "Select a role..."
            </option>
            {move || roles.get().into_iter().map(|(id, name)| {
                let id_str = id.to_string();
                let is_selected = move || selected.get() == Some(id);
                view! {
                    <option value=id_str.clone() selected=is_selected>{name.clone()}</option>
                }
            }).collect_view()}
        </select>
    }
    .into_any()
}
