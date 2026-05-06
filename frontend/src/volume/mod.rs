use std::collections::HashMap;

use filebrowser_types::{BrowseAction, ClientMsg, EntryType};
use yew::prelude::*;
use yew_router::scope_ext::{LocationHandle, RouterScopeExt};

use crate::ws::AppCtx;

// --- View mode ---

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    List,
    Table,
    Icons,
}

impl ViewMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Table => "table",
            Self::Icons => "icons",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "list" => Some(Self::List),
            "table" => Some(Self::Table),
            "icons" => Some(Self::Icons),
            _ => None,
        }
    }
}

fn read_viewmode_cookie() -> Option<ViewMode> {
    let cookies = crate::html_document()?.cookie().ok()?;
    let val = cookies
        .split(';')
        .filter_map(|c| c.trim().strip_prefix("viewmode="))
        .next()?;
    ViewMode::from_str(val)
}

fn write_viewmode_cookie(mode: ViewMode) {
    if let Some(doc) = crate::html_document() {
        let _ = doc.set_cookie(&format!(
            "viewmode={};path=/;max-age=31536000;SameSite=Lax",
            mode.as_str()
        ));
    }
}

// --- File entry helpers ---

fn entry_href(name: &str, is_dir: bool, vid: u64, current_path: &str) -> Option<String> {
    if !is_dir {
        return None;
    }
    let child_path = if current_path == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", current_path.trim_end_matches('/'), name)
    };
    Some(format!("/v/{vid}?path={child_path}"))
}

// --- Component ---

#[derive(Properties, PartialEq)]
pub struct VolumePageProps {
    pub id: u64,
}

pub struct VolumePage {
    ctx: AppCtx,
    _ctx_handle: ContextHandle<AppCtx>,
    _location_handle: LocationHandle,
    current_path: String,
    view_mode: ViewMode,
    /// `seq` of the most recent browse response we have applied. Used to detect
    /// whether the response in context is freshly delivered.
    last_browse_seq: u64,
    loading: bool,
}

pub enum Msg {
    ContextChanged(AppCtx),
    LocationChanged,
    SetViewMode(ViewMode),
}

impl VolumePage {
    fn read_path(ctx: &Context<Self>) -> String {
        ctx.link()
            .location()
            .and_then(|loc| loc.query::<HashMap<String, String>>().ok())
            .and_then(|q| q.get("path").cloned())
            .unwrap_or_else(|| "/".to_string())
    }

    fn request_listing(&self, props: &VolumePageProps) {
        self.ctx
            .send
            .emit(ClientMsg::Browse(BrowseAction::ListDirectory {
                volume_id: props.id,
                path: self.current_path.clone(),
            }));
    }
}

impl Component for VolumePage {
    type Message = Msg;
    type Properties = VolumePageProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (app_ctx, ctx_handle) = ctx
            .link()
            .context::<AppCtx>(ctx.link().callback(Msg::ContextChanged))
            .expect("AppCtx not provided");

        let location_handle = ctx
            .link()
            .add_location_listener(ctx.link().callback(|_| Msg::LocationChanged))
            .expect("router not found");

        let current_path = Self::read_path(ctx);
        let view_mode = read_viewmode_cookie().unwrap_or_default();
        let last_browse_seq = app_ctx.data.browse.seq;
        let mut loading = false;

        if app_ctx.data.ready {
            app_ctx
                .send
                .emit(ClientMsg::Browse(BrowseAction::ListDirectory {
                    volume_id: ctx.props().id,
                    path: current_path.clone(),
                }));
            loading = true;
        }

        Self {
            ctx: app_ctx,
            _ctx_handle: ctx_handle,
            _location_handle: location_handle,
            current_path,
            view_mode,
            last_browse_seq,
            loading,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old: &Self::Properties) -> bool {
        // Volume id changed via routing — refetch.
        if self.ctx.data.ready {
            self.request_listing(ctx.props());
            self.loading = true;
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ContextChanged(c) => {
                let was_not_ready = !self.ctx.data.ready;
                let now_ready = c.data.ready;
                let new_seq = c.data.browse.seq;
                let got_new_browse = new_seq != self.last_browse_seq;
                self.ctx = c;
                if got_new_browse {
                    self.last_browse_seq = new_seq;
                    self.loading = false;
                }
                if was_not_ready && now_ready {
                    self.request_listing(ctx.props());
                    self.loading = true;
                }
                true
            }
            Msg::LocationChanged => {
                let new_path = Self::read_path(ctx);
                if new_path != self.current_path {
                    self.current_path = new_path;
                    if self.ctx.data.ready {
                        self.request_listing(ctx.props());
                        self.loading = true;
                    }
                    return true;
                }
                false
            }
            Msg::SetViewMode(m) => {
                self.view_mode = m;
                write_viewmode_cookie(m);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let vid = ctx.props().id;
        let cp = self.current_path.clone();
        let mode = self.view_mode;

        let volume_name = self
            .ctx
            .data
            .volumes
            .iter()
            .find(|(id, _)| *id == vid)
            .map(|(_, name)| name.clone())
            .unwrap_or_default();

        let on_set_list = ctx.link().callback(|_| Msg::SetViewMode(ViewMode::List));
        let on_set_table = ctx.link().callback(|_| Msg::SetViewMode(ViewMode::Table));
        let on_set_icons = ctx.link().callback(|_| Msg::SetViewMode(ViewMode::Icons));

        let breadcrumbs: Html = {
            let segments: Vec<&str> = cp.split('/').filter(|s| !s.is_empty()).collect();
            let mut crumbs = vec![html! { <li><a href={format!("/v/{vid}")}>{"/"}</a></li> }];
            for (i, seg) in segments.iter().enumerate() {
                let path = format!("/{}", segments[..=i].join("/"));
                let href = format!("/v/{vid}?path={path}");
                crumbs.push(html! { <li><a href={href}>{*seg}</a></li> });
            }
            crumbs.into_iter().collect()
        };

        let error = self.ctx.data.browse.error.as_ref().map(|msg| {
            html! {
                <div class="alert alert-error mb-4">
                    <span>{msg}</span>
                </div>
            }
        });

        let body: Html = if self.loading {
            html! {
                <div class="flex justify-center py-8">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            }
        } else {
            let entries = &self.ctx.data.browse.entries;
            if entries.is_empty() {
                html! { <p class="text-base-content/70">{"This directory is empty."}</p> }
            } else {
                match mode {
                    ViewMode::List => view_list(entries, vid, &cp),
                    ViewMode::Table => view_table(entries, vid, &cp),
                    ViewMode::Icons => view_icons(entries, vid, &cp),
                }
            }
        };

        html! {
            <div class="max-w-4xl mx-auto">
                <div class="flex items-center justify-between mb-2">
                    <h1 class="text-2xl font-bold">{volume_name}</h1>
                    <div class="join">
                        <button
                            class={classes!("btn", "btn-sm", "join-item", (mode == ViewMode::List).then_some("btn-active"))}
                            onclick={on_set_list}
                            title="List"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd"/>
                            </svg>
                        </button>
                        <button
                            class={classes!("btn", "btn-sm", "join-item", (mode == ViewMode::Table).then_some("btn-active"))}
                            onclick={on_set_table}
                            title="Table"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M5 4a3 3 0 00-3 3v6a3 3 0 003 3h10a3 3 0 003-3V7a3 3 0 00-3-3H5zm-1 9v-1h5v2H5a1 1 0 01-1-1zm7 1h4a1 1 0 001-1v-1h-5v2zm0-4h5V8h-5v2zM9 8H4v2h5V8z" clip-rule="evenodd"/>
                            </svg>
                        </button>
                        <button
                            class={classes!("btn", "btn-sm", "join-item", (mode == ViewMode::Icons).then_some("btn-active"))}
                            onclick={on_set_icons}
                            title="Icons"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                <path d="M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                            </svg>
                        </button>
                    </div>
                </div>

                <div class="text-sm breadcrumbs mb-4">
                    <ul>{breadcrumbs}</ul>
                </div>

                {error}
                {body}
            </div>
        }
    }
}

// --- SVG icons ---

fn folder_icon(class: &str) -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class={class.to_string()} viewBox="0 0 20 20" fill="currentColor">
            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
        </svg>
    }
}

fn file_icon(class: &str) -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class={class.to_string()} viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clip-rule="evenodd"/>
        </svg>
    }
}

fn entry_icon(is_dir: bool, class: &str) -> Html {
    if is_dir {
        folder_icon(class)
    } else {
        file_icon(class)
    }
}

fn icon_color(is_dir: bool) -> &'static str {
    if is_dir {
        "h-5 w-5 text-warning"
    } else {
        "h-5 w-5 text-base-content/50"
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if bytes == 0 {
        return "—".into();
    }
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return if size.fract() < 0.05 {
                format!("{:.0} {unit}", size)
            } else {
                format!("{:.1} {unit}", size)
            };
        }
        size /= 1024.0;
    }
    format!("{:.1} PB", size)
}

fn format_datetime(dt: &Option<chrono::DateTime<chrono::Utc>>) -> String {
    match dt {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "—".into(),
    }
}

fn entry_menu() -> Html {
    html! {
        <div class="dropdown dropdown-end">
            <div tabindex="0" role="button" class="btn btn-ghost btn-xs btn-square">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z"/>
                </svg>
            </div>
            <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-10 w-40 p-2 shadow">
                <li><a>{"Download"}</a></li>
                <li><a>{"Rename"}</a></li>
                <li><a class="text-error">{"Delete"}</a></li>
            </ul>
        </div>
    }
}

// --- View modes ---

fn view_list(items: &[filebrowser_types::DirEntry], vid: u64, cp: &str) -> Html {
    let rows: Html = items
        .iter()
        .map(|e| {
            let is_dir = matches!(e.entry_type, EntryType::Directory);
            let href = entry_href(&e.name, is_dir, vid, cp).unwrap_or_default();
            let cls = icon_color(is_dir);
            html! {
                <li>
                    <div class="flex items-center justify-between w-full">
                        <a href={href} class="flex items-center gap-2 flex-1 min-w-0">
                            { entry_icon(is_dir, cls) }
                            <span class="truncate">{&e.name}</span>
                        </a>
                        { entry_menu() }
                    </div>
                </li>
            }
        })
        .collect();
    html! {
        <ul class="menu bg-base-200 rounded-box w-full">{rows}</ul>
    }
}

fn view_table(items: &[filebrowser_types::DirEntry], vid: u64, cp: &str) -> Html {
    let rows: Html = items
        .iter()
        .map(|e| {
            let is_dir = matches!(e.entry_type, EntryType::Directory);
            let href = entry_href(&e.name, is_dir, vid, cp).unwrap_or_default();
            let type_label = if is_dir { "Directory" } else { "File" };
            let size_str = format_size(e.size);
            let created = format_datetime(&e.created_at);
            let updated = format_datetime(&e.updated_at);
            let cls = icon_color(is_dir);
            html! {
                <tr>
                    <td>
                        <a href={href} class="flex items-center gap-2">
                            { entry_icon(is_dir, cls) }
                            <span>{&e.name}</span>
                        </a>
                    </td>
                    <td>{type_label}</td>
                    <td class="text-right">{size_str}</td>
                    <td>{created}</td>
                    <td>{updated}</td>
                    <td class="w-10">{ entry_menu() }</td>
                </tr>
            }
        })
        .collect();
    html! {
        <div>
            <table class="table table-zebra w-full">
                <thead>
                    <tr>
                        <th>{"Name"}</th>
                        <th>{"Type"}</th>
                        <th class="text-right">{"Size"}</th>
                        <th>{"Created"}</th>
                        <th>{"Updated"}</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}

fn view_icons(items: &[filebrowser_types::DirEntry], vid: u64, cp: &str) -> Html {
    let cards: Html = items
        .iter()
        .map(|e| {
            let is_dir = matches!(e.entry_type, EntryType::Directory);
            let href = entry_href(&e.name, is_dir, vid, cp).unwrap_or_default();
            let icon_cls = if is_dir {
                "h-10 w-10 text-warning"
            } else {
                "h-10 w-10 text-base-content/50"
            };
            html! {
                <div class="relative flex flex-col items-center gap-1 p-3 rounded-lg hover:bg-base-200 transition-colors w-28 text-center">
                    <div class="absolute top-1 right-1">{ entry_menu() }</div>
                    <a href={href} class="flex flex-col items-center gap-1">
                        { entry_icon(is_dir, icon_cls) }
                        <span class="text-xs truncate w-full">{&e.name}</span>
                    </a>
                </div>
            }
        })
        .collect();
    html! { <div class="flex flex-wrap gap-2">{cards}</div> }
}
