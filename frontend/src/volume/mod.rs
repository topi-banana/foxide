use filebrowser_types::EntryType;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::ws::WsCtx;

#[derive(Clone)]
struct FileEntry {
    name: String,
    entry_type: EntryType,
}

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
            <h1 class="text-2xl font-bold mb-2">{volume_name}</h1>

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
                if items.is_empty() {
                    view! {
                        <p class="text-base-content/70">"This directory is empty."</p>
                    }.into_any()
                } else {
                    view! {
                        <ul class="menu bg-base-200 rounded-box w-full">
                            {items.into_iter().map(|entry| {
                                let is_dir = matches!(entry.entry_type, EntryType::Directory);
                                let href = if is_dir {
                                    let child_path = if cp == "/" {
                                        format!("/{}", entry.name)
                                    } else {
                                        format!("{}/{}", cp.trim_end_matches('/'), entry.name)
                                    };
                                    Some(format!("/v/{vid}?path={child_path}"))
                                } else {
                                    None
                                };
                                view! {
                                    <li>
                                        <a
                                            href=href.unwrap_or_default()
                                            class="flex items-center gap-2"
                                        >
                                            {if is_dir {
                                                view! {
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-warning" viewBox="0 0 20 20" fill="currentColor">
                                                        <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                                                    </svg>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-base-content/50" viewBox="0 0 20 20" fill="currentColor">
                                                        <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clip-rule="evenodd"/>
                                                    </svg>
                                                }.into_any()
                                            }}
                                            <span>{entry.name}</span>
                                        </a>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }
            })}
        </div>
    }
}
