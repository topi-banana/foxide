use filebrowser_types::EntryType;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::ws::WsCtx;

// --- View mode ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    List,
    Table,
    Icons,
}

impl ViewMode {
    #[allow(dead_code)]
    fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Table => "table",
            Self::Icons => "icons",
        }
    }

    #[allow(dead_code)]
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "list" => Some(Self::List),
            "table" => Some(Self::Table),
            "icons" => Some(Self::Icons),
            _ => None,
        }
    }
}

#[cfg(feature = "hydrate")]
fn read_viewmode_cookie() -> Option<ViewMode> {
    let cookies = crate::html_document()?.cookie().ok()?;
    let val = cookies
        .split(';')
        .filter_map(|c| c.trim().strip_prefix("viewmode="))
        .next()?;
    ViewMode::from_str(val)
}

#[cfg(not(feature = "hydrate"))]
fn read_viewmode_cookie() -> Option<ViewMode> {
    None
}

#[cfg(feature = "hydrate")]
fn write_viewmode_cookie(mode: ViewMode) {
    if let Some(doc) = crate::html_document() {
        let _ = doc.set_cookie(&format!(
            "viewmode={};path=/;max-age=31536000;SameSite=Lax",
            mode.as_str()
        ));
    }
}

#[cfg(not(feature = "hydrate"))]
fn write_viewmode_cookie(_mode: ViewMode) {}

// --- File entry ---

#[derive(Clone)]
struct FileEntry {
    name: String,
    entry_type: EntryType,
    size: u64,
}

impl FileEntry {
    fn is_dir(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory)
    }

    fn href(&self, vid: u64, current_path: &str) -> Option<String> {
        if !self.is_dir() {
            return None;
        }
        let child_path = if current_path == "/" {
            format!("/{}", self.name)
        } else {
            format!("{}/{}", current_path.trim_end_matches('/'), self.name)
        };
        Some(format!("/v/{vid}?path={child_path}"))
    }
}

// --- Component ---

#[component]
pub fn VolumePage() -> impl IntoView {
    let ws = expect_context::<WsCtx>();
    let params = use_params_map();
    let query = use_query_map();

    let volume_id = move || {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<u64>().ok())
    };

    let current_path = move || query.read().get("path").unwrap_or_else(|| "/".to_string());

    let volume_name = move || {
        let vid = volume_id();
        ws.volumes
            .get()
            .into_iter()
            .find(|(id, _)| Some(*id) == vid)
            .map(|(_, name)| name)
            .unwrap_or_default()
    };

    let (entries, set_entries) = signal(Vec::<FileEntry>::new());
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(true);
    let (view_mode, set_view_mode) = signal(ViewMode::List);

    // Restore view mode from cookie on mount
    Effect::new(move |prev: Option<()>| {
        if prev.is_none()
            && let Some(m) = read_viewmode_cookie()
        {
            set_view_mode.set(m);
        }
        write_viewmode_cookie(view_mode.get());
    });

    #[cfg(not(feature = "hydrate"))]
    let _ = (&ws, set_entries, set_error, set_loading);

    #[cfg(feature = "hydrate")]
    {
        use filebrowser_types::BrowseResponse;

        ws.set_on_browse(move |resp| match resp {
            BrowseResponse::DirectoryListing {
                entries: dir_entries,
            } => {
                set_entries.set(
                    dir_entries
                        .into_iter()
                        .map(|e| FileEntry {
                            name: e.name,
                            entry_type: e.entry_type,
                            size: e.size,
                        })
                        .collect(),
                );
                set_error.set(None);
                set_loading.set(false);
            }
            BrowseResponse::Error { message } => {
                set_error.set(Some(message));
                set_loading.set(false);
            }
        });
    }

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        if ws.ready.get()
            && let Some(vid) = volume_id()
        {
            set_loading.set(true);
            ws.send_browse(filebrowser_types::BrowseAction::ListDirectory {
                volume_id: vid,
                path: current_path(),
            });
        }
    });

    view! {
        <div class="max-w-4xl mx-auto">
            <div class="flex items-center justify-between mb-2">
                <h1 class="text-2xl font-bold">{volume_name}</h1>

                // View mode switcher
                <div class="join">
                    <button
                        class="btn btn-sm join-item"
                        class:btn-active={move || view_mode.get() == ViewMode::List}
                        on:click=move |_| set_view_mode.set(ViewMode::List)
                        title="List"
                    >
                        // list icon
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd"/>
                        </svg>
                    </button>
                    <button
                        class="btn btn-sm join-item"
                        class:btn-active={move || view_mode.get() == ViewMode::Table}
                        on:click=move |_| set_view_mode.set(ViewMode::Table)
                        title="Table"
                    >
                        // table icon
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M5 4a3 3 0 00-3 3v6a3 3 0 003 3h10a3 3 0 003-3V7a3 3 0 00-3-3H5zm-1 9v-1h5v2H5a1 1 0 01-1-1zm7 1h4a1 1 0 001-1v-1h-5v2zm0-4h5V8h-5v2zM9 8H4v2h5V8z" clip-rule="evenodd"/>
                        </svg>
                    </button>
                    <button
                        class="btn btn-sm join-item"
                        class:btn-active={move || view_mode.get() == ViewMode::Icons}
                        on:click=move |_| set_view_mode.set(ViewMode::Icons)
                        title="Icons"
                    >
                        // grid icon
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                            <path d="M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                        </svg>
                    </button>
                </div>
            </div>

            <div class="text-sm breadcrumbs mb-4">
                <ul>
                    {move || {
                        let vid = volume_id().unwrap_or(0);
                        let cp = current_path();
                        let segments: Vec<&str> = cp.split('/').filter(|s| !s.is_empty()).collect();
                        let mut crumbs = vec![
                            view! { <li><a href=format!("/v/{vid}")>"/"</a></li> }.into_any(),
                        ];
                        for (i, seg) in segments.iter().enumerate() {
                            let path = format!("/{}", segments[..=i].join("/"));
                            let href = format!("/v/{vid}?path={path}");
                            let seg = seg.to_string();
                            crumbs.push(view! { <li><a href=href>{seg}</a></li> }.into_any());
                        }
                        crumbs
                    }}
                </ul>
            </div>

            {move || error.get().map(|msg| view! {
                <div class="alert alert-error mb-4">
                    <span>{msg}</span>
                </div>
            })}

            {move || loading.get().then(|| view! {
                <div class="flex justify-center py-8">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            })}

            {move || (!loading.get()).then(|| {
                let items = entries.get();
                let vid = volume_id().unwrap_or(0);
                let cp = current_path();
                let mode = view_mode.get();
                if items.is_empty() {
                    view! {
                        <p class="text-base-content/70">"This directory is empty."</p>
                    }.into_any()
                } else {
                    match mode {
                        ViewMode::List => view_list(items, vid, &cp),
                        ViewMode::Table => view_table(items, vid, &cp),
                        ViewMode::Icons => view_icons(items, vid, &cp),
                    }
                }
            })}
        </div>
    }
}

// --- SVG icons ---

fn folder_icon(class: &'static str) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 20 20" fill="currentColor">
            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
        </svg>
    }
}

fn file_icon(class: &'static str) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clip-rule="evenodd"/>
        </svg>
    }
}

fn entry_icon(is_dir: bool, class: &'static str) -> impl IntoView {
    if is_dir {
        folder_icon(class).into_any()
    } else {
        file_icon(class).into_any()
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

// --- List view ---

fn view_list(items: Vec<FileEntry>, vid: u64, cp: &str) -> AnyView {
    let rows: Vec<_> = items
        .into_iter()
        .map(|entry| {
            let is_dir = entry.is_dir();
            let href = entry.href(vid, cp);
            let cls = icon_color(is_dir);
            view! {
                <li>
                    <a href=href.unwrap_or_default() class="flex items-center gap-2">
                        {entry_icon(is_dir, cls)}
                        <span>{entry.name}</span>
                    </a>
                </li>
            }
        })
        .collect();
    view! {
        <ul class="menu bg-base-200 rounded-box w-full">
            {rows}
        </ul>
    }
    .into_any()
}

// --- Table view ---

fn view_table(items: Vec<FileEntry>, vid: u64, cp: &str) -> AnyView {
    let rows: Vec<_> = items
        .into_iter()
        .map(|entry| {
            let is_dir = entry.is_dir();
            let href = entry.href(vid, cp);
            let type_label = if is_dir { "Directory" } else { "File" };
            let size_str = format_size(entry.size);
            let cls = icon_color(is_dir);
            view! {
                <tr>
                    <td>
                        <a href=href.unwrap_or_default() class="flex items-center gap-2">
                            {entry_icon(is_dir, cls)}
                            <span>{entry.name}</span>
                        </a>
                    </td>
                    <td>{type_label}</td>
                    <td class="text-right">{size_str}</td>
                </tr>
            }
        })
        .collect();
    view! {
        <div class="overflow-x-auto">
            <table class="table table-zebra w-full">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Type"</th>
                        <th class="text-right">"Size"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
    .into_any()
}

// --- Icons view ---

fn view_icons(items: Vec<FileEntry>, vid: u64, cp: &str) -> AnyView {
    let cards: Vec<_> = items
        .into_iter()
        .map(|entry| {
            let is_dir = entry.is_dir();
            let href = entry.href(vid, cp).unwrap_or_default();
            let icon_cls = if is_dir {
                "h-10 w-10 text-warning"
            } else {
                "h-10 w-10 text-base-content/50"
            };
            view! {
                <a
                    href=href
                    class="flex flex-col items-center gap-1 p-3 rounded-lg hover:bg-base-200 transition-colors w-28 text-center"
                >
                    {entry_icon(is_dir, icon_cls)}
                    <span class="text-xs truncate w-full">{entry.name}</span>
                </a>
            }
        })
        .collect();
    view! {
        <div class="flex flex-wrap gap-2">
            {cards}
        </div>
    }
    .into_any()
}
