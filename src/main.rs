use leptos::*;
use uuid::Uuid;
use std::rc::Rc;
use gloo_timers::callback::Interval;
use serde::{Deserialize, Serialize};

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResourceKind { Link, Note, Doc }
impl ResourceKind {
    fn label(&self) -> &'static str { match self { ResourceKind::Link=>"Link", ResourceKind::Note=>"Note", ResourceKind::Doc=>"Doc" } }
    fn icon(&self) -> &'static str { match self { ResourceKind::Link=>"🔗", ResourceKind::Note=>"📝", ResourceKind::Doc=>"📄" } }
    fn from_str(s: &str) -> Self { match s { "Note"=>ResourceKind::Note, "Doc"=>ResourceKind::Doc, _=>ResourceKind::Link } }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    pub id:    String,
    pub title: String,
    pub url:   String,
    pub kind:  ResourceKind,
    pub notes: String,
}
impl Resource {
    fn new(title: &str, url: &str, kind: ResourceKind, notes: &str) -> Self {
        Resource {
            id:    Uuid::new_v4().to_string(),
            title: title.to_string(),
            url:   url.to_string(),
            kind,
            notes: notes.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TagDef {
    pub id:    String,
    pub name:  String,
    pub color: String,
}

impl TagDef {
    fn new(id: &str, name: &str, color: &str) -> Self {
        TagDef { id: id.to_string(), name: name.to_string(), color: color.to_string() }
    }
    fn badge_style(&self) -> String {
        format!("background:{}28;color:{};border:1px solid {}55;padding:2px 8px;border-radius:5px;font-size:0.72rem;font-weight:600;", self.color, self.color, self.color)
    }
}

fn initial_tags() -> Vec<TagDef> {
    vec![
        TagDef::new("feature", "Feature", "#3b82f6"),
        TagDef::new("bug",     "Bug",     "#ef4444"),
        TagDef::new("task",    "Task",    "#22c55e"),
        TagDef::new("design",  "Design",  "#a855f7"),
    ]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Space {
    pub id:    String,
    pub name:  String,
    pub color: String,
    pub desc:  String,
}

fn initial_spaces() -> Vec<Space> {
    vec![
        Space { id: "space-1".to_string(), name: "Engineering".to_string(), color: "#6366f1".to_string(), desc: "Core platform engineering work".to_string() },
    ]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subtask {
    pub id:        String,
    pub title:     String,
    pub done:      bool,
    pub resources: Vec<Resource>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Story {
    pub id:                 String,
    pub epic_id:            String,
    pub title:              String,
    pub direct_logged_secs: u64,
    pub resources:          Vec<Resource>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Epic {
    pub id:                 String,
    pub title:              String,
    pub desc:               String,
    pub color:              String,
    pub deadline:           String,
    pub estimated_hours:    f32,
    pub direct_logged_secs: u64,
    pub resources:          Vec<Resource>,
    pub space_id:           Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub id:          String,
    pub title:       String,
    pub desc:        String,
    pub tag:         String,
    pub epic_id:     Option<String>,
    pub story_id:    Option<String>,
    pub subtasks:    Vec<Subtask>,
    pub logged_secs: u64,
    pub resources:   Vec<Resource>,
}

impl Card {
    fn new(title: &str, desc: &str, tag: &str) -> Self {
        Card {
            id:          Uuid::new_v4().to_string(),
            title:       title.to_string(),
            desc:        desc.to_string(),
            tag:         tag.to_string(),
            epic_id:     None,
            story_id:    None,
            subtasks:    vec![],
            logged_secs: 0,
            resources:   vec![],
        }
    }

    fn new_full(title: &str, desc: &str, tag: &str, epic_id: Option<String>, story_id: Option<String>) -> Self {
        Card {
            id:          Uuid::new_v4().to_string(),
            title:       title.to_string(),
            desc:        desc.to_string(),
            tag:         tag.to_string(),
            epic_id,
            story_id,
            subtasks:    vec![],
            logged_secs: 0,
            resources:   vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub id:    String,
    pub title: String,
    pub cards: Vec<Card>,
}

// ── App state helpers ─────────────────────────────────────────────────────────

fn initial_columns() -> Vec<Column> {
    vec![
        Column {
            id:    "todo".to_string(),
            title: "To Do".to_string(),
            cards: vec![
                Card::new("Design system tokens",  "Define color / spacing tokens for the design system.", "design"),
                Card::new("Auth flow",              "Implement OAuth2 login + refresh token rotation.",    "feature"),
                Card::new("Fix nav overflow bug",  "Mobile nav overflows viewport at <375px.",            "bug"),
            ],
        },
        Column {
            id:    "in-progress".to_string(),
            title: "In Progress".to_string(),
            cards: vec![
                Card::new("Kanban WASM POC", "Rust + Leptos kanban that compiles to WebAssembly.", "task"),
            ],
        },
        Column {
            id:    "review".to_string(),
            title: "Review".to_string(),
            cards: vec![
                Card::new("API rate limiting", "Add sliding-window rate limiter to REST endpoints.", "feature"),
            ],
        },
        Column {
            id:    "done".to_string(),
            title: "Done".to_string(),
            cards: vec![
                Card::new("Repo setup", "Init monorepo, CI pipeline, branch protection rules.", "task"),
            ],
        },
    ]
}

fn initial_epics() -> Vec<Epic> {
    vec![
        Epic {
            id:                 "epic-1".to_string(),
            title:              "Platform v2".to_string(),
            desc:               "Full platform rewrite".to_string(),
            color:              "#6366f1".to_string(),
            deadline:           "2026-06-30".to_string(),
            estimated_hours:    120.0,
            direct_logged_secs: 0,
            resources:          vec![],
            space_id:           Some("space-1".to_string()),
        },
        Epic {
            id:                 "epic-2".to_string(),
            title:              "Design System".to_string(),
            desc:               "Component library and tokens".to_string(),
            color:              "#f59e0b".to_string(),
            deadline:           "2026-04-15".to_string(),
            estimated_hours:    40.0,
            direct_logged_secs: 0,
            resources:          vec![],
            space_id:           Some("space-1".to_string()),
        },
    ]
}

// ── Aggregation helpers ───────────────────────────────────────────────────────

#[allow(dead_code)]
fn task_total_secs(card: &Card) -> u64 { card.logged_secs }

fn story_total_secs(story: &Story, columns: &[Column]) -> u64 {
    story.direct_logged_secs
    + columns.iter().flat_map(|c| &c.cards)
        .filter(|c| c.story_id.as_deref() == Some(story.id.as_str()))
        .map(|c| c.logged_secs)
        .sum::<u64>()
}

fn epic_total_secs(epic: &Epic, stories: &[Story], columns: &[Column]) -> u64 {
    epic.direct_logged_secs
    + stories.iter()
        .filter(|s| s.epic_id == epic.id)
        .map(|s| story_total_secs(s, columns))
        .sum::<u64>()
    // Cards on the epic but not assigned to any story
    + columns.iter()
        .flat_map(|c| &c.cards)
        .filter(|c| c.epic_id.as_deref() == Some(epic.id.as_str()) && c.story_id.is_none())
        .map(|c| c.logged_secs)
        .sum::<u64>()
}

fn fmt_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
}

// ── Persistence ───────────────────────────────────────────────────────────────

const STORAGE_KEY: &str = "kanban_state_v1";

#[derive(Serialize, Deserialize, Clone)]
struct SavedState {
    columns:   Vec<Column>,
    epics:     Vec<Epic>,
    stories:   Vec<Story>,
    tags:      Vec<TagDef>,
    spaces:    Vec<Space>,
    nav_items: Vec<(String, String)>,
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn save_state(state: &SavedState) {
    if let Some(s) = storage() {
        if let Ok(json) = serde_json::to_string(state) {
            let _ = s.set_item(STORAGE_KEY, &json);
        }
    }
}

fn load_state() -> Option<SavedState> {
    let json = storage()?.get_item(STORAGE_KEY).ok()??;
    serde_json::from_str(&json).ok()
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    let saved = load_state();

    let (columns, set_columns)               = create_signal(saved.as_ref().map(|s| s.columns.clone()).unwrap_or_else(initial_columns));
    let (dragging, set_dragging)             = create_signal::<Option<(String, String)>>(None);
    let (show_modal, set_show_modal)         = create_signal(false);
    let (modal_col, set_modal_col)           = create_signal("todo".to_string());
    let (show_col_modal, set_show_col_modal) = create_signal(false);

    // column drag-to-reorder
    let (dragging_col, set_dragging_col)     = create_signal::<Option<String>>(None);
    let (col_drag_over, set_col_drag_over)   = create_signal::<Option<String>>(None);

    let (epics, set_epics)                   = create_signal(saved.as_ref().map(|s| s.epics.clone()).unwrap_or_else(initial_epics));
    let (stories, set_stories)               = create_signal(saved.as_ref().map(|s| s.stories.clone()).unwrap_or_default());
    let (tags, set_tags)                     = create_signal(saved.as_ref().map(|s| s.tags.clone()).unwrap_or_else(initial_tags));
    let (spaces, set_spaces)                 = create_signal(saved.as_ref().map(|s| s.spaces.clone()).unwrap_or_else(initial_spaces));
    let (active_view, set_active_view)       = create_signal("board".to_string());
    // nav tabs drag-to-reorder
    let default_nav = vec![
        ("Board".to_string(),     "board".to_string()),
        ("Dashboard".to_string(), "dashboard".to_string()),
        ("Resources".to_string(), "resources".to_string()),
    ];
    let (nav_items, set_nav_items)           = create_signal(saved.as_ref().map(|s| s.nav_items.clone()).unwrap_or(default_nav));
    let (nav_drag_src, set_nav_drag_src)     = create_signal::<Option<String>>(None);
    let (nav_drag_over_k, set_nav_drag_over_k) = create_signal::<Option<String>>(None);
    // ("task"|"story"|"epic", id)
    let (timer_target, set_timer_target)     = create_signal(Option::<(String,String)>::None);
    let (show_epic_modal, set_show_epic_modal)   = create_signal(false);
    let (show_story_modal, set_show_story_modal) = create_signal(false);
    let (story_epic_id, set_story_epic_id)       = create_signal(String::new());
    let (show_space_modal, set_show_space_modal) = create_signal(false);
    let (show_tag_modal, set_show_tag_modal)     = create_signal(false);

    let (show_res_modal, set_show_res_modal) = create_signal(false);
    // target for the resource modal: ("card"|"epic"|"story", id)
    let (res_target, set_res_target)         = create_signal(Option::<(String,String)>::None);
    // dashboard level tabs: "spaces" | "epics" | "stories" | "tasks" | "tags"
    let (dashboard_level, set_dashboard_level) = create_signal("spaces");
    // edit card modal
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (edit_card_id, set_edit_card_id)       = create_signal(String::new());

    // ── Pomodoro timer signals ─────────────────────────────────────────────────
    let (timer_secs, set_timer_secs)       = create_signal(25u32 * 60);
    let (timer_running, set_timer_running) = create_signal(false);
    let (timer_mode, set_timer_mode)       = create_signal("Work");

    let interval_handle: StoredValue<Option<Interval>> = store_value(None::<Interval>);

    // ── Auto-save to localStorage ─────────────────────────────────────────────
    create_effect(move |_| {
        let state = SavedState {
            columns:   columns.get(),
            epics:     epics.get(),
            stories:   stories.get(),
            tags:      tags.get(),
            spaces:    spaces.get(),
            nav_items: nav_items.get(),
        };
        save_state(&state);
    });

    // ── drag handlers ─────────────────────────────────────────────────────────

    let on_drag_start = move |card_id: String, col_id: String| {
        set_dragging.set(Some((card_id, col_id)));
    };

    let on_drop = move |target_col_id: String| {
        if let Some((card_id, src_col_id)) = dragging.get() {
            if src_col_id == target_col_id {
                set_dragging.set(None);
                return;
            }
            set_columns.update(|cols| {
                let card = cols
                    .iter_mut()
                    .find(|c| c.id == src_col_id)
                    .and_then(|c| {
                        c.cards.iter().position(|k| k.id == card_id).map(|i| c.cards.remove(i))
                    });
                if let Some(card) = card {
                    if let Some(col) = cols.iter_mut().find(|c| c.id == target_col_id) {
                        col.cards.push(card);
                    }
                }
            });
            set_dragging.set(None);
        }
    };

    // ── delete handler ────────────────────────────────────────────────────────

    let on_delete = move |card_id: String| {
        set_columns.update(|cols| {
            for col in cols.iter_mut() {
                if let Some(i) = col.cards.iter().position(|c| c.id == card_id) {
                    col.cards.remove(i);
                    break;
                }
            }
        });
    };

    // ── edit card handler ─────────────────────────────────────────────────────

    let on_edit_card = move |card_id: String, title: String, desc: String, tag: String, epic_id: Option<String>, story_id: Option<String>, resources: Vec<Resource>| {
        set_columns.update(|cols| {
            for col in cols.iter_mut() {
                if let Some(card) = col.cards.iter_mut().find(|c| c.id == card_id) {
                    card.title    = title.clone();
                    card.desc     = desc.clone();
                    card.tag      = tag.clone();
                    card.epic_id  = epic_id.clone();
                    card.story_id = story_id.clone();
                    card.resources = resources.clone();
                    break;
                }
            }
        });
    };

    // ── add card handler ──────────────────────────────────────────────────────

    let on_add_card = move |col_id: String, title: String, desc: String, tag: String, epic_id: Option<String>, story_id: Option<String>| {
        set_columns.update(|cols| {
            if let Some(col) = cols.iter_mut().find(|c| c.id == col_id) {
                col.cards.push(Card::new_full(&title, &desc, &tag, epic_id, story_id));
            }
        });
    };

    // ── add column handler ────────────────────────────────────────────────────

    let on_add_column = move |name: String| {
        set_columns.update(|cols| {
            cols.push(Column {
                id:    Uuid::new_v4().to_string(),
                title: name,
                cards: vec![],
            });
        });
    };

    let on_delete_column = move |col_id: String| {
        set_columns.update(|cols| cols.retain(|c| c.id != col_id));
    };

    // ── column reorder handlers ───────────────────────────────────────────────

    let on_col_drag_start = move |col_id: String| {
        set_dragging_col.set(Some(col_id));
    };

    let on_col_drag_over = move |col_id: String| {
        set_col_drag_over.set(Some(col_id));
    };

    let on_col_drop = move |target_col_id: String| {
        if let Some(src_id) = dragging_col.get() {
            if src_id != target_col_id {
                set_columns.update(|cols| {
                    if let Some(src_idx) = cols.iter().position(|c| c.id == src_id) {
                        if let Some(tgt_idx) = cols.iter().position(|c| c.id == target_col_id) {
                            let col = cols.remove(src_idx);
                            let insert_at = if src_idx < tgt_idx { tgt_idx } else { tgt_idx };
                            cols.insert(insert_at, col);
                        }
                    }
                });
            }
        }
        set_dragging_col.set(None);
        set_col_drag_over.set(None);
    };

    let on_col_drag_end = move |_| {
        set_dragging_col.set(None);
        set_col_drag_over.set(None);
    };

    // ── epic & story handlers ─────────────────────────────────────────────────

    let on_add_epic = move |title: String, desc: String, color: String, deadline: String, est: f32, space_id: Option<String>| {
        set_epics.update(|epics| {
            epics.push(Epic {
                id:                 Uuid::new_v4().to_string(),
                title,
                desc,
                color,
                deadline,
                estimated_hours:    est,
                direct_logged_secs: 0,
                resources:          vec![],
                space_id,
            });
        });
    };

    let on_add_space = move |name: String, color: String, desc: String| {
        set_spaces.update(|spaces| {
            spaces.push(Space { id: Uuid::new_v4().to_string(), name, color, desc });
        });
    };

    let on_delete_space = move |space_id: String| {
        // Unassign epics from this space rather than deleting them
        set_epics.update(|epics| {
            for e in epics.iter_mut() {
                if e.space_id.as_deref() == Some(space_id.as_str()) {
                    e.space_id = None;
                }
            }
        });
        set_spaces.update(|spaces| spaces.retain(|s| s.id != space_id));
    };

    let on_add_tag = move |name: String, color: String| {
        let id = name.to_lowercase().replace(' ', "-");
        set_tags.update(|tags| {
            if !tags.iter().any(|t| t.id == id) {
                tags.push(TagDef::new(&id, &name, &color));
            }
        });
    };

    let on_delete_tag = move |tag_id: String| {
        set_tags.update(|tags| tags.retain(|t| t.id != tag_id));
    };

    let on_delete_epic = move |epic_id: String| {
        // Collect story IDs belonging to this epic so we can cascade-delete their tasks too
        let story_ids_to_delete: Vec<String> = stories.get()
            .into_iter()
            .filter(|s| s.epic_id == epic_id)
            .map(|s| s.id)
            .collect();

        // Delete all cards assigned to this epic OR any of its stories
        set_columns.update(|cols| {
            for col in cols.iter_mut() {
                col.cards.retain(|card| {
                    card.epic_id.as_deref() != Some(epic_id.as_str())
                        && !card.story_id.as_ref().map(|sid| story_ids_to_delete.contains(sid)).unwrap_or(false)
                });
            }
        });
        // Delete all stories belonging to this epic
        set_stories.update(|stories| {
            stories.retain(|s| s.epic_id != epic_id);
        });
        // Delete the epic itself
        set_epics.update(|epics| {
            if let Some(i) = epics.iter().position(|e| e.id == epic_id) {
                epics.remove(i);
            }
        });
    };

    let on_delete_story = move |story_id: String| {
        // Delete all cards (tasks) assigned to this story
        set_columns.update(|cols| {
            for col in cols.iter_mut() {
                col.cards.retain(|card| {
                    card.story_id.as_deref() != Some(story_id.as_str())
                });
            }
        });
        // Delete the story itself
        set_stories.update(|stories| {
            stories.retain(|s| s.id != story_id);
        });
    };

    let on_add_story = move |epic_id: String, title: String| {
        set_stories.update(|stories| {
            stories.push(Story {
                id:                 Uuid::new_v4().to_string(),
                epic_id,
                title,
                direct_logged_secs: 0,
                resources:          vec![],
            });
        });
    };

    // ── timer handlers ────────────────────────────────────────────────────────

    let timer_toggle = move |_| {
        if timer_running.get() {
            interval_handle.update_value(|v| { *v = None; });
            set_timer_running.set(false);
        } else {
            set_timer_running.set(true);
            let interval = Interval::new(1000, move || {
                set_timer_secs.update(|s| {
                    if *s > 0 {
                        *s -= 1;
                        if timer_mode.get() == "Work" {
                            if let Some((kind, id)) = timer_target.get() {
                                match kind.as_str() {
                                    "task" => set_columns.update(|cols| {
                                        for col in cols.iter_mut() {
                                            if let Some(card) = col.cards.iter_mut().find(|c| c.id == id) {
                                                card.logged_secs += 1; break;
                                            }
                                        }
                                    }),
                                    "story" => set_stories.update(|stories| {
                                        if let Some(s) = stories.iter_mut().find(|s| s.id == id) {
                                            s.direct_logged_secs += 1;
                                        }
                                    }),
                                    "epic" => set_epics.update(|epics| {
                                        if let Some(e) = epics.iter_mut().find(|e| e.id == id) {
                                            e.direct_logged_secs += 1;
                                        }
                                    }),
                                    _ => {}
                                }
                            }
                        }
                    }
                    if *s == 0 {
                        interval_handle.update_value(|v| { *v = None; });
                        set_timer_running.set(false);
                        let next_mode = if timer_mode.get() == "Work" { "Break" } else { "Work" };
                        set_timer_mode.set(next_mode);
                        *s = if next_mode == "Work" { 25 * 60 } else { 5 * 60 };
                    }
                });
            });
            interval_handle.set_value(Some(interval));
        }
    };

    let timer_reset = move |_| {
        interval_handle.update_value(|v| { *v = None; });
        set_timer_running.set(false);
        set_timer_mode.set("Work");
        set_timer_secs.set(25 * 60);
    };

    let timer_skip = move |_| {
        interval_handle.update_value(|v| { *v = None; });
        set_timer_running.set(false);
        let next_mode = if timer_mode.get() == "Work" { "Break" } else { "Work" };
        set_timer_mode.set(next_mode);
        set_timer_secs.set(if next_mode == "Work" { 25 * 60 } else { 5 * 60 });
    };

    view! {
        <div>
            // ── Header ──────────────────────────────────────────────────────
            <header class="header">
                <h1>"Kanban Board"</h1>

                <nav class="view-tabs">
                    <For
                        each=move || nav_items.get()
                        key=|item| item.1.clone()
                        children=move |item| {
                            let label = item.0.clone();
                            let key   = item.1.clone();
                            let k1 = key.clone();
                            let k2 = key.clone();
                            let k3 = key.clone();
                            let k4 = key.clone();
                            let k5 = key.clone();
                            view! {
                                <button
                                    class=move || {
                                        let mut c = if active_view.get() == k1 { "tab active".to_string() } else { "tab".to_string() };
                                        if nav_drag_over_k.get().as_deref() == Some(k1.as_str()) { c.push_str(" nav-drag-over"); }
                                        c
                                    }
                                    draggable="true"
                                    on:click=move |_| set_active_view.set(k2.clone())
                                    on:dragstart=move |_| set_nav_drag_src.set(Some(k3.clone()))
                                    on:dragover=move |ev| { ev.prevent_default(); set_nav_drag_over_k.set(Some(k4.clone())); }
                                    on:dragleave=move |_| set_nav_drag_over_k.set(None)
                                    on:drop=move |ev| {
                                        ev.prevent_default();
                                        if let Some(src) = nav_drag_src.get() {
                                            if src != k5 {
                                                set_nav_items.update(|items| {
                                                    if let (Some(si), Some(ti)) = (
                                                        items.iter().position(|(_, k)| *k == src),
                                                        items.iter().position(|(_, k)| *k == k5),
                                                    ) {
                                                        let itm = items.remove(si);
                                                        let at  = if si < ti { ti } else { ti };
                                                        items.insert(at, itm);
                                                    }
                                                });
                                            }
                                        }
                                        set_nav_drag_src.set(None);
                                        set_nav_drag_over_k.set(None);
                                    }
                                >{ label }</button>
                            }
                        }
                    />
                </nav>

                // ── Pomodoro Timer ───────────────────────────────────────────
                <div class="timer">
                    <span class="timer-mode">{ move || timer_mode.get() }</span>
                    <select
                        class="timer-epic-select"
                        prop:value=move || timer_target.get().map(|(k,id)| format!("{}:{}", k, id)).unwrap_or_default()
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            if v.is_empty() {
                                set_timer_target.set(None);
                            } else if let Some(pos) = v.find(':') {
                                let kind = v[..pos].to_string();
                                let id   = v[pos+1..].to_string();
                                set_timer_target.set(Some((kind, id)));
                            }
                        }
                    >
                        <option value="">"-- none --"</option>
                        <optgroup label="Epics">
                            { move || epics.get().into_iter().map(|ep| {
                                let val   = format!("epic:{}", ep.id);
                                let title = ep.title.clone();
                                view! { <option value=val>{ title }</option> }
                            }).collect_view() }
                        </optgroup>
                        <optgroup label="Stories">
                            { move || stories.get().into_iter().map(|s| {
                                let val   = format!("story:{}", s.id);
                                let title = s.title.clone();
                                view! { <option value=val>{ title }</option> }
                            }).collect_view() }
                        </optgroup>
                        <optgroup label="Tasks">
                            { move || columns.get().into_iter().flat_map(|col| col.cards.into_iter()).map(|card| {
                                let val   = format!("task:{}", card.id);
                                let title = card.title.clone();
                                view! { <option value=val>{ title }</option> }
                            }).collect_view() }
                        </optgroup>
                    </select>
                    <span class="timer-display">
                        { move || {
                            let s = timer_secs.get();
                            format!("{:02}:{:02}", s / 60, s % 60)
                        }}
                    </span>
                    <button class="timer-btn" on:click=timer_toggle>
                        { move || if timer_running.get() { "⏸" } else { "▶" } }
                    </button>
                    <button class="timer-btn" on:click=timer_reset>"↺"</button>
                    <button class="timer-btn" on:click=timer_skip>"⏭"</button>
                </div>

                <Show when=move || active_view.get() == "board">
                    <button
                        class="add-card-btn"
                        on:click=move |_| {
                            set_modal_col.set("todo".to_string());
                            set_show_modal.set(true);
                        }
                    >
                        "+ New Card"
                    </button>
                </Show>

                <Show when=move || active_view.get() == "dashboard">
                    <button
                        class="add-card-btn"
                        on:click=move |_| set_show_epic_modal.set(true)
                    >
                        "+ New Epic"
                    </button>
                </Show>
            </header>

            // ── Board view ───────────────────────────────────────────────────
            <Show when=move || active_view.get() == "board">
                <div class="board">
                    <For
                        each=move || columns.get()
                        key=|col| col.id.clone()
                        children=move |col| {
                            let col_id              = col.id.clone();
                            let col_id_drop         = col_id.clone();
                            let col_id_add          = col_id.clone();
                            let col_id_modal        = col_id.clone();
                            let col_id_for_card     = col_id.clone();
                            let col_id_reorder_drag = col_id.clone();
                            let col_id_reorder_over = col_id.clone();
                            let col_id_reorder_drop = col_id.clone();
                            let col_id_del          = col_id.clone();
                            let on_drop_c             = on_drop.clone();
                            let on_delete_c           = on_delete.clone();
                            let on_drag_start_c       = on_drag_start.clone();
                            let on_col_drag_start_c   = on_col_drag_start.clone();
                            let on_col_drag_over_c    = on_col_drag_over.clone();
                            let on_col_drop_c         = on_col_drop.clone();
                            let on_col_drag_end_c     = on_col_drag_end.clone();
                            let on_delete_column_c    = on_delete_column.clone();

                            let (drag_over, set_drag_over) = create_signal(false);

                            let title_class = match col.id.as_str() {
                                "todo"        => "column-title todo",
                                "in-progress" => "column-title in-prog",
                                "review"      => "column-title review",
                                "done"        => "column-title done",
                                _             => "column-title",
                            };

                            view! {
                                <div
                                    class=move || {
                                        let over = col_drag_over.get().as_deref() == Some(col_id_reorder_over.as_str());
                                        if drag_over.get() { "column drag-over".to_string() }
                                        else if over { "column col-drag-target".to_string() }
                                        else { "column".to_string() }
                                    }
                                    on:dragover=move |ev| {
                                        ev.prevent_default();
                                        // Only set card drag-over when a card is being dragged
                                        if dragging_col.get().is_some() {
                                            on_col_drag_over_c(col_id_reorder_drop.clone());
                                        } else {
                                            set_drag_over.set(true);
                                        }
                                    }
                                    on:dragleave=move |_| { set_drag_over.set(false); }
                                    on:drop=move |ev| {
                                        ev.prevent_default();
                                        set_drag_over.set(false);
                                        if dragging_col.get().is_some() {
                                            on_col_drop_c(col_id_drop.clone());
                                        } else {
                                            on_drop_c(col_id_drop.clone());
                                        }
                                    }
                                >
                                    <div class="column-header">
                                        <div class="column-header-left">
                                            <span
                                                class="col-drag-handle"
                                                draggable="true"
                                                on:dragstart=move |ev| {
                                                    ev.stop_propagation();
                                                    on_col_drag_start_c(col_id_reorder_drag.clone());
                                                }
                                                on:dragend=move |ev| on_col_drag_end_c(ev)
                                            >
                                                "⠿"
                                            </span>
                                            <span class=title_class>{ col.title.clone() }</span>
                                        </div>
                                        <div style="display:flex;align-items:center;gap:6px;">
                                            <span class="column-count">
                                                { move || {
                                                    columns.get()
                                                        .iter()
                                                        .find(|c| c.id == col_id)
                                                        .map(|c| c.cards.len())
                                                        .unwrap_or(0)
                                                }}
                                            </span>
                                            <button
                                                class="col-delete-btn"
                                                title="Delete column"
                                                on:click=move |_| on_delete_column_c(col_id_del.clone())
                                            >"✕"</button>
                                        </div>
                                    </div>

                                    <div class="column-body">
                                        <For
                                            each=move || {
                                                columns.get()
                                                    .into_iter()
                                                    .find(|c| c.id == col_id_add)
                                                    .map(|c| c.cards)
                                                    .unwrap_or_default()
                                            }
                                            key=|card| card.id.clone()
                                            children=move |card| {
                                                let card_id_drag       = card.id.clone();
                                                let card_id_del        = card.id.clone();
                                                let card_id_res        = card.id.clone();
                                                let card_id_edit       = card.id.clone();
                                                let src_col            = col_id_for_card.clone();
                                                let on_drag_start_card = on_drag_start_c.clone();
                                                let on_delete_card     = on_delete_c.clone();
                                                let card_tag_id        = card.tag.clone();
                                                let card_epic_id       = card.epic_id.clone();
                                                let subtask_done       = card.subtasks.iter().filter(|s| s.done).count();
                                                let subtask_total      = card.subtasks.len();
                                                let card_resources     = card.resources.clone();

                                                let border_style = move || {
                                                    if let Some(ref eid) = card_epic_id {
                                                        let eid_clone = eid.clone();
                                                        let color = epics.get()
                                                            .into_iter()
                                                            .find(|e| e.id == eid_clone)
                                                            .map(|e| e.color.clone())
                                                            .unwrap_or_default();
                                                        if color.is_empty() {
                                                            String::new()
                                                        } else {
                                                            format!("border-left: 3px solid {};", color)
                                                        }
                                                    } else {
                                                        String::new()
                                                    }
                                                };

                                                view! {
                                                    <div
                                                        class="card"
                                                        style=border_style
                                                        draggable="true"
                                                        on:dragstart=move |_| {
                                                            on_drag_start_card(card_id_drag.clone(), src_col.clone());
                                                        }
                                                    >
                                                        <div class="card-title">{ &card.title }</div>
                                                        <div class="card-desc">{ &card.desc }</div>
                                                        { if subtask_total > 0 {
                                                            view! {
                                                                <div class="subtask-progress">
                                                                    { format!("{}/{} subtasks", subtask_done, subtask_total) }
                                                                </div>
                                                            }.into_view()
                                                        } else {
                                                            view! { <span></span> }.into_view()
                                                        }}
                                                        <Show when={let cr = card_resources.clone(); move || !cr.is_empty()}>
                                                            <div class="card-resources">
                                                                { card_resources.iter().map(|res| {
                                                                    let url   = res.url.clone();
                                                                    let icon  = res.kind.icon();
                                                                    let title = res.title.clone();
                                                                    view! {
                                                                        <a class="resource-chip-sm" href=url target="_blank">
                                                                            { icon }{" "}{ title }
                                                                        </a>
                                                                    }
                                                                }).collect_view() }
                                                            </div>
                                                        </Show>
                                                        <button
                                                            class="add-resource-btn"
                                                            on:click=move |_| {
                                                                set_res_target.set(Some(("card".to_string(), card_id_res.clone())));
                                                                set_show_res_modal.set(true);
                                                            }
                                                        >
                                                            "＋ resource"
                                                        </button>
                                                        <div class="card-footer">
                                                            { move || {
                                                                let tid  = card_tag_id.clone();
                                                                let tdef = tags.get();
                                                                let t    = tdef.iter().find(|t| t.id == tid);
                                                                let nm   = t.map(|t| t.name.clone()).unwrap_or_else(|| tid.clone());
                                                                let st   = t.map(|t| t.badge_style()).unwrap_or_default();
                                                                view! { <span class="card-tag" style=st>{ nm }</span> }
                                                            }}
                                                            <div class="card-actions">
                                                                <button
                                                                    class="card-edit-btn"
                                                                    title="Edit card"
                                                                    on:click=move |_| {
                                                                        set_edit_card_id.set(card_id_edit.clone());
                                                                        set_show_edit_modal.set(true);
                                                                    }
                                                                >"✎"</button>
                                                                <button
                                                                    class="card-delete"
                                                                    on:click=move |_| {
                                                                        on_delete_card(card_id_del.clone());
                                                                    }
                                                                >
                                                                    "✕"
                                                                </button>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />

                                        {
                                            let cid = col_id_modal.clone();
                                            view! {
                                                <button
                                                    class="add-to-column-btn"
                                                    on:click=move |_| {
                                                        set_modal_col.set(cid.clone());
                                                        set_show_modal.set(true);
                                                    }
                                                >
                                                    "+ Add card"
                                                </button>
                                            }
                                        }
                                    </div>
                                </div>
                            }
                        }
                    />

                    <button
                        class="add-column-btn"
                        on:click=move |_| set_show_col_modal.set(true)
                    >
                        "+ Add column"
                    </button>
                </div>
            </Show>

            // ── Dashboard view ───────────────────────────────────────────────
            <Show when=move || active_view.get() == "dashboard">
                <div class="dashboard">
                    <div class="dashboard-header">
                        <h2>"Dashboard"</h2>
                        <div class="dashboard-level-tabs">
                            <button
                                class=move || if dashboard_level.get() == "spaces" { "level-tab active" } else { "level-tab" }
                                on:click=move |_| set_dashboard_level.set("spaces")
                            >"Spaces"</button>
                            <button
                                class=move || if dashboard_level.get() == "epics" { "level-tab active" } else { "level-tab" }
                                on:click=move |_| set_dashboard_level.set("epics")
                            >"Epics"</button>
                            <button
                                class=move || if dashboard_level.get() == "stories" { "level-tab active" } else { "level-tab" }
                                on:click=move |_| set_dashboard_level.set("stories")
                            >"Stories"</button>
                            <button
                                class=move || if dashboard_level.get() == "tasks" { "level-tab active" } else { "level-tab" }
                                on:click=move |_| set_dashboard_level.set("tasks")
                            >"Tasks"</button>
                            <button
                                class=move || if dashboard_level.get() == "tags" { "level-tab active" } else { "level-tab" }
                                on:click=move |_| set_dashboard_level.set("tags")
                            >"Tags"</button>
                        </div>
                        <Show when=move || dashboard_level.get() == "spaces">
                            <button class="add-card-btn" on:click=move |_| set_show_space_modal.set(true)>"+ New Space"</button>
                        </Show>
                        <Show when=move || dashboard_level.get() == "epics">
                            <button
                                class="add-card-btn"
                                on:click=move |_| set_show_epic_modal.set(true)
                            >
                                "+ New Epic"
                            </button>
                        </Show>
                        <Show when=move || dashboard_level.get() == "tags">
                            <button class="add-card-btn" on:click=move |_| set_show_tag_modal.set(true)>"+ New Tag"</button>
                        </Show>
                    </div>
                    <Show when=move || dashboard_level.get() == "epics">
                    <div class="epic-grid">
                        <For
                            each=move || epics.get()
                            key=|e| e.id.clone()
                            children=move |epic| {
                                let epic_id_prog_p  = epic.id.clone();
                                let epic_id_prog_l  = epic.id.clone();
                                let epic_id_stories = epic.id.clone();
                                let epic_id_story   = epic.id.clone();
                                let epic_id_res     = epic.id.clone();
                                let epic_id_del     = epic.id.clone();
                                let epic_color      = epic.color.clone();
                                let epic_title      = epic.title.clone();
                                let epic_desc       = epic.desc.clone();
                                let epic_deadline   = epic.deadline.clone();
                                let est_h           = epic.estimated_hours;
                                let epic_resources  = epic.resources.clone();
                                let overdue_class   = if epic.deadline.as_str() < "2026-03-30" { "epic-deadline overdue" } else { "epic-deadline" };
                                let color_bar_style = format!("background:{};", epic_color);

                                let epic_id_timer  = epic.id.clone();
                                let eid_total      = epic.id.clone();
                                let total_secs_fn  = move || {
                                    let stories_snap = stories.get();
                                    let cols_snap    = columns.get();
                                    let epics_snap   = epics.get();
                                    epics_snap.iter()
                                        .find(|e| e.id == eid_total)
                                        .map(|e| epic_total_secs(e, &stories_snap, &cols_snap))
                                        .unwrap_or(0)
                                };
                                let eid_direct     = epic.id.clone();
                                let direct_secs_fn = move || {
                                    epics.get()
                                        .into_iter()
                                        .find(|e| e.id == eid_direct)
                                        .map(|e| e.direct_logged_secs)
                                        .unwrap_or(0)
                                };

                                let pct = move || {
                                    let eid_t = epic_id_prog_p.clone();
                                    let t = columns.get()
                                        .iter()
                                        .flat_map(|c| c.cards.clone())
                                        .filter(|card| card.epic_id.as_deref() == Some(eid_t.as_str()))
                                        .count();
                                    let eid_d = epic_id_prog_p.clone();
                                    let d = columns.get()
                                        .iter()
                                        .find(|c| c.id == "done")
                                        .map(|c| c.cards.iter().filter(|card| card.epic_id.as_deref() == Some(eid_d.as_str())).count())
                                        .unwrap_or(0);
                                    if t > 0 { d * 100 / t } else { 0 }
                                };
                                let progress_label = move || {
                                    let eid_t = epic_id_prog_l.clone();
                                    let t = columns.get()
                                        .iter()
                                        .flat_map(|c| c.cards.clone())
                                        .filter(|card| card.epic_id.as_deref() == Some(eid_t.as_str()))
                                        .count();
                                    let eid_d = epic_id_prog_l.clone();
                                    let d = columns.get()
                                        .iter()
                                        .find(|c| c.id == "done")
                                        .map(|c| c.cards.iter().filter(|card| card.epic_id.as_deref() == Some(eid_d.as_str())).count())
                                        .unwrap_or(0);
                                    format!("{}/{} tasks", d, t)
                                };

                                view! {
                                    <div class="epic-card">
                                        <div class="epic-color-bar" style=color_bar_style></div>
                                        <div class="epic-body">
                                            <div class="epic-top-row">
                                                <span class="epic-title">{ epic_title }</span>
                                                <div style="display:flex;align-items:center;gap:8px;">
                                                    <span class=overdue_class>{ epic_deadline }</span>
                                                    <button
                                                        class="timer-start-btn"
                                                        title="Start timer for this epic"
                                                        on:click=move |_| set_timer_target.set(Some(("epic".to_string(), epic_id_timer.clone())))
                                                    >"▶"</button>
                                                    <button
                                                        class="epic-delete-btn"
                                                        title="Delete epic"
                                                        on:click=move |_| on_delete_epic(epic_id_del.clone())
                                                    >
                                                        "✕"
                                                    </button>
                                                </div>
                                            </div>
                                            <p class="epic-desc">{ epic_desc }</p>
                                            <div class="epic-progress-row">
                                                <div class="progress-bar-bg">
                                                    <div
                                                        class="progress-bar-fill"
                                                        style=move || format!("width:{}%;", pct())
                                                    ></div>
                                                </div>
                                                <span class="progress-label">
                                                    { progress_label }
                                                </span>
                                            </div>
                                            <div class="epic-time-row">
                                                <span class="epic-time-total">{ move || format!("Total: {}", fmt_duration(total_secs_fn())) }</span>
                                                <span class="time-sep">" · "</span>
                                                <span class="time-direct">{ move || format!("Direct: {}", fmt_duration(direct_secs_fn())) }</span>
                                                <span class="time-sep">" · "</span>
                                                <span class="time-est">{ format!("Est: {}h", est_h) }</span>
                                            </div>
                                            <div class="epic-stories">
                                                <For
                                                    each=move || {
                                                        let eid = epic_id_stories.clone();
                                                        stories.get()
                                                            .into_iter()
                                                            .filter(move |s| s.epic_id == eid)
                                                            .collect::<Vec<_>>()
                                                    }
                                                    key=|s| s.id.clone()
                                                    children=move |story| {
                                                        let story_id_res   = story.id.clone();
                                                        let story_id_del   = story.id.clone();
                                                        let story_id_time  = story.id.clone();
                                                        let story_id_timer = story.id.clone();
                                                        let story_time = move || {
                                                            let cols  = columns.get();
                                                            let slist = stories.get();
                                                            slist.iter()
                                                                .find(|s| s.id == story_id_time)
                                                                .map(|s| story_total_secs(s, &cols))
                                                                .unwrap_or(0)
                                                        };
                                                        view! {
                                                            <div class="story-chip-row">
                                                                <div class="story-chip">{ story.title.clone() }</div>
                                                                <span class="story-chip-time">{ move || fmt_duration(story_time()) }</span>
                                                                <button
                                                                    class="timer-start-btn"
                                                                    title="Start timer for this story"
                                                                    on:click=move |_| set_timer_target.set(Some(("story".to_string(), story_id_timer.clone())))
                                                                >"▶"</button>
                                                                <button
                                                                    class="add-resource-btn"
                                                                    title="Add resource to story"
                                                                    on:click=move |_| {
                                                                        set_res_target.set(Some(("story".to_string(), story_id_res.clone())));
                                                                        set_show_res_modal.set(true);
                                                                    }
                                                                >
                                                                    "＋"
                                                                </button>
                                                                <button
                                                                    class="story-delete-btn"
                                                                    title="Delete story and its tasks"
                                                                    on:click=move |_| on_delete_story(story_id_del.clone())
                                                                >
                                                                    "✕"
                                                                </button>
                                                            </div>
                                                        }
                                                    }
                                                />
                                                <button
                                                    class="add-story-btn"
                                                    on:click=move |_| {
                                                        set_story_epic_id.set(epic_id_story.clone());
                                                        set_show_story_modal.set(true);
                                                    }
                                                >
                                                    "+ Story"
                                                </button>
                                            </div>
                                            { if !epic_resources.is_empty() {
                                                let res_list = epic_resources.clone();
                                                view! {
                                                    <div class="resource-list">
                                                        { res_list.into_iter().map(|res| {
                                                            let url   = res.url.clone();
                                                            let icon  = res.kind.icon();
                                                            let title = res.title.clone();
                                                            view! {
                                                                <a class="resource-chip-sm" href=url target="_blank">
                                                                    { icon }{" "}{ title }
                                                                </a>
                                                            }
                                                        }).collect_view() }
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! { <span></span> }.into_view()
                                            }}
                                            <button
                                                class="add-resource-btn"
                                                on:click=move |_| {
                                                    set_res_target.set(Some(("epic".to_string(), epic_id_res.clone())));
                                                    set_show_res_modal.set(true);
                                                }
                                            >
                                                "＋ resource"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                    </Show>

                    <Show when=move || dashboard_level.get() == "stories">
                        <div class="level-list">
                            { move || {
                                let slist = stories.get();
                                if slist.is_empty() {
                                    view! { <div class="resource-empty">"No stories yet. Add stories inside epics."</div> }.into_view()
                                } else {
                                    slist.into_iter().map(|story| {
                                        let sid        = story.id.clone();
                                        let sid_timer  = story.id.clone();
                                        let sid2       = sid.clone();
                                        let epic_color = epics.get().into_iter().find(|e| e.id == story.epic_id).map(|e| e.color.clone()).unwrap_or_default();
                                        let epic_name  = epics.get().into_iter().find(|e| e.id == story.epic_id).map(|e| e.title.clone()).unwrap_or_default();
                                        let task_count = columns.get().iter().flat_map(|c| c.cards.clone()).filter(|c| c.story_id.as_deref() == Some(sid.as_str())).count();
                                        let done_count = columns.get().iter().find(|c| c.id == "done").map(|c| c.cards.iter().filter(|card| card.story_id.as_deref() == Some(sid.as_str())).count()).unwrap_or(0);
                                        let story_time = move || { let cols = columns.get(); let sl = stories.get(); sl.iter().find(|s| s.id == sid2).map(|s| story_total_secs(s, &cols)).unwrap_or(0) };
                                        let direct_secs = story.direct_logged_secs;
                                        let badge_style = format!("background:{}22;color:{};border:1px solid {}44;", epic_color, epic_color, epic_color);
                                        view! {
                                            <div class="level-row">
                                                <div class="level-row-left">
                                                    <span class="level-row-title">{ story.title.clone() }</span>
                                                    <span class="level-epic-badge" style=badge_style>{ epic_name }</span>
                                                    <span class="level-row-tasks">{ format!("{}/{} tasks", done_count, task_count) }</span>
                                                </div>
                                                <div class="level-row-right">
                                                    <span class="level-time">{ move || fmt_duration(story_time()) }</span>
                                                    <span class="level-time-label">"total"</span>
                                                    <span class="level-time-direct">{ format!("direct: {}", fmt_duration(direct_secs)) }</span>
                                                    <button
                                                        class="timer-start-btn"
                                                        title="Start timer for this story"
                                                        on:click=move |_| set_timer_target.set(Some(("story".to_string(), sid_timer.clone())))
                                                    >"▶"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view().into_view()
                                }
                            }}
                        </div>
                    </Show>

                    <Show when=move || dashboard_level.get() == "tasks">
                        <div class="level-list">
                            { move || {
                                let cols = columns.get();
                                let all_cards: Vec<(Card, String)> = cols.into_iter()
                                    .flat_map(|col| { let ct = col.title.clone(); col.cards.into_iter().map(move |card| (card, ct.clone())) })
                                    .collect();
                                if all_cards.is_empty() {
                                    view! { <div class="resource-empty">"No tasks yet."</div> }.into_view()
                                } else {
                                    all_cards.into_iter().map(|(card, col_title)| {
                                        let cid       = card.id.clone();
                                        let cid_timer = card.id.clone();
                                        let epic_name = card.epic_id.as_ref()
                                            .and_then(|eid| epics.get().into_iter().find(|e| &e.id == eid))
                                            .map(|e| e.title.clone())
                                            .unwrap_or_default();
                                        let story_name = card.story_id.as_ref()
                                            .and_then(|sid| stories.get().into_iter().find(|s| &s.id == sid))
                                            .map(|s| s.title.clone())
                                            .unwrap_or_default();
                                        let task_time = move || { columns.get().into_iter().flat_map(|c| c.cards.into_iter()).find(|c| c.id == cid).map(|c| c.logged_secs).unwrap_or(0) };
                                        view! {
                                            <div class="level-row">
                                                <div class="level-row-left">
                                                    <span class="level-row-title">{ card.title.clone() }</span>
                                                    <span class="level-col-badge">{ col_title.clone() }</span>
                                                    { if !epic_name.is_empty() { view! { <span class="level-epic-ref">{ epic_name }</span> }.into_view() } else { view! { <span></span> }.into_view() }}
                                                    { if !story_name.is_empty() { view! { <span class="level-story-ref">{ story_name }</span> }.into_view() } else { view! { <span></span> }.into_view() }}
                                                </div>
                                                <div class="level-row-right">
                                                    <span class="level-time">{ move || fmt_duration(task_time()) }</span>
                                                    <button
                                                        class="timer-start-btn"
                                                        title="Start timer for this task"
                                                        on:click=move |_| set_timer_target.set(Some(("task".to_string(), cid_timer.clone())))
                                                    >"▶"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view().into_view()
                                }
                            }}
                        </div>
                    </Show>

                    <Show when=move || dashboard_level.get() == "spaces">
                        <div class="level-list">
                            { move || {
                                let slist = spaces.get();
                                if slist.is_empty() {
                                    view! { <div class="resource-empty">"No spaces yet."</div> }.into_view()
                                } else {
                                    slist.into_iter().map(|space| {
                                        let sid      = space.id.clone();
                                        let sid_del  = space.id.clone();
                                        let epic_count = epics.get().iter().filter(|e| e.space_id.as_deref() == Some(sid.as_str())).count();
                                        let total_time = {
                                            let sid2 = sid.clone();
                                            move || {
                                                let eps = epics.get();
                                                let sts = stories.get();
                                                let cls = columns.get();
                                                eps.iter()
                                                    .filter(|e| e.space_id.as_deref() == Some(sid2.as_str()))
                                                    .map(|e| epic_total_secs(e, &sts, &cls))
                                                    .sum::<u64>()
                                            }
                                        };
                                        let bar_style = format!("background:{};", space.color);
                                        view! {
                                            <div class="space-row">
                                                <div class="space-color-dot" style=bar_style></div>
                                                <div class="space-row-info">
                                                    <span class="space-row-name">{ space.name.clone() }</span>
                                                    <span class="space-row-desc">{ space.desc.clone() }</span>
                                                </div>
                                                <div class="level-row-right">
                                                    <span class="level-row-tasks">{ format!("{} epics", epic_count) }</span>
                                                    <span class="level-time">{ move || fmt_duration(total_time()) }</span>
                                                    <button
                                                        class="epic-delete-btn"
                                                        title="Delete space"
                                                        on:click=move |_| on_delete_space(sid_del.clone())
                                                    >"✕"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view().into_view()
                                }
                            }}
                        </div>
                    </Show>

                    <Show when=move || dashboard_level.get() == "tags">
                        <div class="level-list">
                            { move || {
                                let tlist = tags.get();
                                tlist.into_iter().map(|t| {
                                    let tid     = t.id.clone();
                                    let tid_del = t.id.clone();
                                    let badge   = t.badge_style();
                                    let card_count = columns.get().iter().flat_map(|c| c.cards.iter().map(|k| k.tag.clone()).collect::<Vec<_>>()).filter(|k| *k == tid).count();
                                    view! {
                                        <div class="level-row">
                                            <div class="level-row-left">
                                                <span class="card-tag" style=badge>{ t.name.clone() }</span>
                                                <span class="level-row-tasks">{ format!("{} cards", card_count) }</span>
                                                <span class="tag-id-label">{ format!("id: {}", tid.clone()) }</span>
                                            </div>
                                            <div class="level-row-right">
                                                <div class="tag-color-swatch" style=format!("background:{};", t.color)></div>
                                                <span class="tag-color-val">{ t.color.clone() }</span>
                                                <button
                                                    class="epic-delete-btn"
                                                    title="Delete tag"
                                                    on:click=move |_| on_delete_tag(tid_del.clone())
                                                >"✕"</button>
                                            </div>
                                        </div>
                                    }
                                }).collect_view().into_view()
                            }}
                        </div>
                    </Show>
                </div>
            </Show>

            // ── Resources view ───────────────────────────────────────────────
            <Show when=move || active_view.get() == "resources">
                <div class="resources-view">
                    <div class="resources-header">
                        <h2>"Resources"</h2>
                    </div>
                    <div class="resource-sections">
                        // Epics section
                        <div>
                            <div class="resource-section-title">"Epics"</div>
                            { move || {
                                let epics_list = epics.get();
                                let has_any = epics_list.iter().any(|e| !e.resources.is_empty());
                                if !has_any {
                                    view! { <div class="resource-empty">"No resources yet."</div> }.into_view()
                                } else {
                                    epics_list.into_iter()
                                        .filter(|e| !e.resources.is_empty())
                                        .map(|e| {
                                            let name      = e.title.clone();
                                            let resources = e.resources.clone();
                                            view! {
                                                <div class="resource-entity">
                                                    <div class="resource-entity-name">{ name }</div>
                                                    <div class="resource-list">
                                                        { resources.into_iter().map(|res| {
                                                            let url        = res.url.clone();
                                                            let icon       = res.kind.icon();
                                                            let title      = res.title.clone();
                                                            let kind_label = res.kind.label();
                                                            view! {
                                                                <a class="resource-chip" href=url target="_blank">
                                                                    { icon }{" "}{ title }
                                                                    <span class="resource-kind">{ kind_label }</span>
                                                                </a>
                                                            }
                                                        }).collect_view() }
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view().into_view()
                                }
                            }}
                        </div>
                        // Stories section
                        <div>
                            <div class="resource-section-title">"Stories"</div>
                            { move || {
                                let stories_list = stories.get();
                                let has_any = stories_list.iter().any(|s| !s.resources.is_empty());
                                if !has_any {
                                    view! { <div class="resource-empty">"No resources yet."</div> }.into_view()
                                } else {
                                    stories_list.into_iter()
                                        .filter(|s| !s.resources.is_empty())
                                        .map(|s| {
                                            let name      = s.title.clone();
                                            let resources = s.resources.clone();
                                            view! {
                                                <div class="resource-entity">
                                                    <div class="resource-entity-name">{ name }</div>
                                                    <div class="resource-list">
                                                        { resources.into_iter().map(|res| {
                                                            let url        = res.url.clone();
                                                            let icon       = res.kind.icon();
                                                            let title      = res.title.clone();
                                                            let kind_label = res.kind.label();
                                                            view! {
                                                                <a class="resource-chip" href=url target="_blank">
                                                                    { icon }{" "}{ title }
                                                                    <span class="resource-kind">{ kind_label }</span>
                                                                </a>
                                                            }
                                                        }).collect_view() }
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view().into_view()
                                }
                            }}
                        </div>
                        // Tasks section
                        <div>
                            <div class="resource-section-title">"Tasks"</div>
                            { move || {
                                let cols = columns.get();
                                let all_cards: Vec<Card> = cols.into_iter().flat_map(|c| c.cards.into_iter()).collect();
                                let has_any = all_cards.iter().any(|c| !c.resources.is_empty());
                                if !has_any {
                                    view! { <div class="resource-empty">"No resources yet."</div> }.into_view()
                                } else {
                                    all_cards.into_iter()
                                        .filter(|c| !c.resources.is_empty())
                                        .map(|c| {
                                            let name      = c.title.clone();
                                            let resources = c.resources.clone();
                                            view! {
                                                <div class="resource-entity">
                                                    <div class="resource-entity-name">{ name }</div>
                                                    <div class="resource-list">
                                                        { resources.into_iter().map(|res| {
                                                            let url        = res.url.clone();
                                                            let icon       = res.kind.icon();
                                                            let title      = res.title.clone();
                                                            let kind_label = res.kind.label();
                                                            view! {
                                                                <a class="resource-chip" href=url target="_blank">
                                                                    { icon }{" "}{ title }
                                                                    <span class="resource-kind">{ kind_label }</span>
                                                                </a>
                                                            }
                                                        }).collect_view() }
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view().into_view()
                                }
                            }}
                        </div>
                    </div>
                </div>
            </Show>

            // ── New card modal ────────────────────────────────────────────────
            <Show when=move || show_modal.get()>
                <AddCardModal
                    col_id=modal_col
                    columns=columns
                    epics=epics
                    stories=stories
                    tags=tags
                    on_save=move |(col_id, title, desc, tag, epic_id_str, story_id_str)| {
                        let eid = if epic_id_str.is_empty() { None } else { Some(epic_id_str) };
                        let sid = if story_id_str.is_empty() { None } else { Some(story_id_str) };
                        on_add_card(col_id, title, desc, tag, eid, sid);
                        set_show_modal.set(false);
                    }
                    on_cancel=move || set_show_modal.set(false)
                />
            </Show>

            // ── New column modal ──────────────────────────────────────────────
            <Show when=move || show_col_modal.get()>
                <AddColumnModal
                    on_save=move |name| {
                        on_add_column(name);
                        set_show_col_modal.set(false);
                    }
                    on_cancel=move || set_show_col_modal.set(false)
                />
            </Show>

            // ── New epic modal ────────────────────────────────────────────────
            <Show when=move || show_epic_modal.get()>
                <AddEpicModal
                    spaces=spaces
                    on_save=move |title, desc, color, deadline, est, space_id| {
                        on_add_epic(title, desc, color, deadline, est, space_id);
                        set_show_epic_modal.set(false);
                    }
                    on_cancel=move || set_show_epic_modal.set(false)
                />
            </Show>

            // ── New space modal ───────────────────────────────────────────────
            <Show when=move || show_space_modal.get()>
                <AddSpaceModal
                    on_save=move |name, color, desc| {
                        on_add_space(name, color, desc);
                        set_show_space_modal.set(false);
                    }
                    on_cancel=move || set_show_space_modal.set(false)
                />
            </Show>

            // ── New tag modal ─────────────────────────────────────────────────
            <Show when=move || show_tag_modal.get()>
                <AddTagModal
                    on_save=move |name, color| {
                        on_add_tag(name, color);
                        set_show_tag_modal.set(false);
                    }
                    on_cancel=move || set_show_tag_modal.set(false)
                />
            </Show>

            // ── New story modal ───────────────────────────────────────────────
            <Show when=move || show_story_modal.get()>
                <AddStoryModal
                    epics=epics
                    initial_epic_id=story_epic_id
                    on_save=move |epic_id, title| {
                        on_add_story(epic_id, title);
                        set_show_story_modal.set(false);
                    }
                    on_cancel=move || set_show_story_modal.set(false)
                />
            </Show>

            // ── Edit card modal ───────────────────────────────────────────────
            <Show when=move || show_edit_modal.get()>
                <EditCardModal
                    card_id=edit_card_id
                    columns=columns
                    epics=epics
                    stories=stories
                    tags=tags
                    on_save=move |(card_id, title, desc, tag, epic_id_str, story_id_str, resources)| {
                        let eid = if epic_id_str.is_empty() { None } else { Some(epic_id_str) };
                        let sid = if story_id_str.is_empty() { None } else { Some(story_id_str) };
                        on_edit_card(card_id, title, desc, tag, eid, sid, resources);
                        set_show_edit_modal.set(false);
                    }
                    on_cancel=move || set_show_edit_modal.set(false)
                />
            </Show>

            // ── Add resource modal ────────────────────────────────────────────
            <Show when=move || show_res_modal.get()>
                <AddResourceModal
                    on_save=move |(title, url, kind, notes)| {
                        if let Some((entity_type, id)) = res_target.get() {
                            let res = Resource::new(&title, &url, kind, &notes);
                            match entity_type.as_str() {
                                "card" => set_columns.update(|cols| {
                                    for col in cols.iter_mut() {
                                        if let Some(c) = col.cards.iter_mut().find(|c| c.id == id) {
                                            c.resources.push(res.clone()); break;
                                        }
                                    }
                                }),
                                "epic" => set_epics.update(|epics| {
                                    if let Some(e) = epics.iter_mut().find(|e| e.id == id) {
                                        e.resources.push(res.clone());
                                    }
                                }),
                                "story" => set_stories.update(|stories| {
                                    if let Some(s) = stories.iter_mut().find(|s| s.id == id) {
                                        s.resources.push(res.clone());
                                    }
                                }),
                                _ => {}
                            }
                        }
                        set_show_res_modal.set(false);
                    }
                    on_cancel=move || set_show_res_modal.set(false)
                />
            </Show>
        </div>
    }
}

// ── AddCardModal component ────────────────────────────────────────────────────

#[component]
fn AddCardModal(
    col_id: ReadSignal<String>,
    columns: ReadSignal<Vec<Column>>,
    epics: ReadSignal<Vec<Epic>>,
    stories: ReadSignal<Vec<Story>>,
    tags: ReadSignal<Vec<TagDef>>,
    on_save: impl Fn((String, String, String, String, String, String)) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (title, set_title)                   = create_signal(String::new());
    let (desc, set_desc)                     = create_signal(String::new());
    let (tag, set_tag)                       = create_signal("feature".to_string());
    let (selected_col, set_selected_col)     = create_signal(col_id.get_untracked());
    let (selected_epic, set_selected_epic)   = create_signal(String::new());
    let (selected_story, set_selected_story) = create_signal(String::new());

    create_effect(move |_| {
        set_selected_col.set(col_id.get());
    });

    let on_cancel_overlay = on_cancel.clone();
    let on_cancel_btn     = on_cancel.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| on_cancel_overlay()>
            <div class="modal" on:click=|ev| ev.stop_propagation()>
                <h2>"New Card"</h2>

                <div class="form-group">
                    <label>"Title"</label>
                    <input
                        type="text"
                        placeholder="Card title…"
                        prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Description"</label>
                    <textarea
                        placeholder="Optional description…"
                        prop:value=desc
                        on:input=move |ev| set_desc.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Column"</label>
                    <select
                        prop:value=selected_col
                        on:change=move |ev| set_selected_col.set(event_target_value(&ev))
                    >
                        { move || columns.get().into_iter().map(|col| {
                            let id     = col.id.clone();
                            let ctitle = col.title.clone();
                            view! { <option value=id>{ ctitle }</option> }
                        }).collect_view() }
                    </select>
                </div>

                <div class="form-group">
                    <label>"Tag"</label>
                    <select prop:value=tag on:change=move |ev| set_tag.set(event_target_value(&ev))>
                        { move || tags.get().into_iter().map(|t| {
                            let tid = t.id.clone(); let tn = t.name.clone();
                            view! { <option value=tid>{ tn }</option> }
                        }).collect_view() }
                    </select>
                </div>

                <div class="form-group">
                    <label>"Epic"</label>
                    <select
                        prop:value=selected_epic
                        on:change=move |ev| {
                            set_selected_epic.set(event_target_value(&ev));
                            set_selected_story.set(String::new());
                        }
                    >
                        <option value="">"None"</option>
                        { move || epics.get().into_iter().map(|ep| {
                            let eid    = ep.id.clone();
                            let etitle = ep.title.clone();
                            view! { <option value=eid>{ etitle }</option> }
                        }).collect_view() }
                    </select>
                </div>

                <div class="form-group">
                    <label>"Story"</label>
                    <select
                        prop:value=selected_story
                        on:change=move |ev| set_selected_story.set(event_target_value(&ev))
                    >
                        <option value="">"None"</option>
                        { move || {
                            let cur_epic = selected_epic.get();
                            stories.get()
                                .into_iter()
                                .filter(move |s| s.epic_id == cur_epic)
                                .map(|s| {
                                    let sid    = s.id.clone();
                                    let stitle = s.title.clone();
                                    view! { <option value=sid>{ stitle }</option> }
                                })
                                .collect_view()
                        }}
                    </select>
                </div>

                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| on_cancel_btn()>"Cancel"</button>
                    <button
                        class="btn-create"
                        on:click=move |_| {
                            let t = title.get();
                            if !t.trim().is_empty() {
                                on_save((
                                    selected_col.get(),
                                    t,
                                    desc.get(),
                                    tag.get(),
                                    selected_epic.get(),
                                    selected_story.get(),
                                ));
                            }
                        }
                    >
                        "Create"
                    </button>
                </div>
            </div>
        </div>
    }
}

// ── AddColumnModal component ──────────────────────────────────────────────────

#[component]
fn AddColumnModal(
    on_save: impl Fn(String) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (col_name, set_col_name) = create_signal(String::new());

    let on_cancel_overlay = on_cancel.clone();
    let on_cancel_btn     = on_cancel.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| on_cancel_overlay()>
            <div class="modal" on:click=|ev| ev.stop_propagation()>
                <h2>"New Column"</h2>

                <div class="form-group">
                    <label>"Column name"</label>
                    <input
                        type="text"
                        placeholder="Column name…"
                        prop:value=col_name
                        on:input=move |ev| set_col_name.set(event_target_value(&ev))
                    />
                </div>

                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| on_cancel_btn()>"Cancel"</button>
                    <button
                        class="btn-create"
                        on:click=move |_| {
                            let n = col_name.get();
                            if !n.trim().is_empty() {
                                on_save(n);
                            }
                        }
                    >
                        "Create"
                    </button>
                </div>
            </div>
        </div>
    }
}

// ── AddEpicModal component ────────────────────────────────────────────────────

#[component]
fn AddEpicModal(
    spaces:   ReadSignal<Vec<Space>>,
    on_save: impl Fn(String, String, String, String, f32, Option<String>) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (title, set_title)         = create_signal(String::new());
    let (desc, set_desc)           = create_signal(String::new());
    let (color, set_color)         = create_signal("#6366f1".to_string());
    let (deadline, set_deadline)   = create_signal(String::new());
    let (est_hours, set_est_hours) = create_signal(8.0f32);
    let (space_id, set_space_id)   = create_signal(String::new());

    let on_cancel_overlay = on_cancel.clone();
    let on_cancel_btn     = on_cancel.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| on_cancel_overlay()>
            <div class="modal" on:click=|ev| ev.stop_propagation()>
                <h2>"New Epic"</h2>

                <div class="form-group">
                    <label>"Title"</label>
                    <input
                        type="text"
                        placeholder="Epic title…"
                        prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Description"</label>
                    <textarea
                        placeholder="Epic description…"
                        prop:value=desc
                        on:input=move |ev| set_desc.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Color"</label>
                    <select
                        prop:value=color
                        on:change=move |ev| set_color.set(event_target_value(&ev))
                    >
                        <option value="#6366f1">"Indigo"</option>
                        <option value="#f59e0b">"Amber"</option>
                        <option value="#10b981">"Emerald"</option>
                        <option value="#f43f5e">"Rose"</option>
                        <option value="#0ea5e9">"Sky"</option>
                        <option value="#8b5cf6">"Violet"</option>
                    </select>
                </div>

                <div class="form-group">
                    <label>"Deadline"</label>
                    <input
                        type="date"
                        prop:value=deadline
                        on:input=move |ev| set_deadline.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Estimated Hours"</label>
                    <input
                        type="number"
                        min="1"
                        step="0.5"
                        prop:value=move || est_hours.get().to_string()
                        on:input=move |ev| {
                            let v = event_target_value(&ev);
                            if let Ok(f) = v.parse::<f32>() {
                                set_est_hours.set(f);
                            }
                        }
                    />
                </div>

                <div class="form-group">
                    <label>"Space"</label>
                    <select prop:value=space_id on:change=move |ev| set_space_id.set(event_target_value(&ev))>
                        <option value="">"None"</option>
                        { move || spaces.get().into_iter().map(|s| {
                            let sid = s.id.clone(); let sn = s.name.clone();
                            view! { <option value=sid>{ sn }</option> }
                        }).collect_view() }
                    </select>
                </div>

                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| on_cancel_btn()>"Cancel"</button>
                    <button
                        class="btn-create"
                        on:click=move |_| {
                            let t = title.get();
                            if !t.trim().is_empty() {
                                let sid = space_id.get();
                                let sid_opt = if sid.is_empty() { None } else { Some(sid) };
                                on_save(t, desc.get(), color.get(), deadline.get(), est_hours.get(), sid_opt);
                            }
                        }
                    >
                        "Create"
                    </button>
                </div>
            </div>
        </div>
    }
}

// ── AddStoryModal component ───────────────────────────────────────────────────

#[component]
fn AddStoryModal(
    epics: ReadSignal<Vec<Epic>>,
    initial_epic_id: ReadSignal<String>,
    on_save: impl Fn(String, String) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (selected_epic, set_selected_epic) = create_signal(initial_epic_id.get_untracked());
    let (story_title, set_story_title)     = create_signal(String::new());

    create_effect(move |_| {
        set_selected_epic.set(initial_epic_id.get());
    });

    let on_cancel_overlay = on_cancel.clone();
    let on_cancel_btn     = on_cancel.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| on_cancel_overlay()>
            <div class="modal" on:click=|ev| ev.stop_propagation()>
                <h2>"New Story"</h2>

                <div class="form-group">
                    <label>"Epic"</label>
                    <select
                        prop:value=selected_epic
                        on:change=move |ev| set_selected_epic.set(event_target_value(&ev))
                    >
                        { move || epics.get().into_iter().map(|ep| {
                            let eid    = ep.id.clone();
                            let etitle = ep.title.clone();
                            view! { <option value=eid>{ etitle }</option> }
                        }).collect_view() }
                    </select>
                </div>

                <div class="form-group">
                    <label>"Story Title"</label>
                    <input
                        type="text"
                        placeholder="Story title…"
                        prop:value=story_title
                        on:input=move |ev| set_story_title.set(event_target_value(&ev))
                    />
                </div>

                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| on_cancel_btn()>"Cancel"</button>
                    <button
                        class="btn-create"
                        on:click=move |_| {
                            let t = story_title.get();
                            if !t.trim().is_empty() {
                                on_save(selected_epic.get(), t);
                            }
                        }
                    >
                        "Create"
                    </button>
                </div>
            </div>
        </div>
    }
}

// ── AddResourceModal component ────────────────────────────────────────────────

#[component]
fn AddResourceModal(
    on_save: impl Fn((String, String, ResourceKind, String)) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (title, set_title) = create_signal(String::new());
    let (url, set_url)     = create_signal(String::new());
    let (kind, set_kind)   = create_signal("Link".to_string());
    let (notes, set_notes) = create_signal(String::new());

    let on_cancel_overlay = on_cancel.clone();
    let on_cancel_btn     = on_cancel.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| on_cancel_overlay()>
            <div class="modal" on:click=|ev| ev.stop_propagation()>
                <h2>"Add Resource"</h2>

                <div class="form-group">
                    <label>"Title"</label>
                    <input
                        type="text"
                        placeholder="Resource title…"
                        prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"URL"</label>
                    <input
                        type="text"
                        placeholder="https://…"
                        prop:value=url
                        on:input=move |ev| set_url.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Kind"</label>
                    <select
                        prop:value=kind
                        on:change=move |ev| set_kind.set(event_target_value(&ev))
                    >
                        <option value="Link">"Link"</option>
                        <option value="Note">"Note"</option>
                        <option value="Doc">"Doc"</option>
                    </select>
                </div>

                <div class="form-group">
                    <label>"Notes (optional)"</label>
                    <textarea
                        placeholder="Optional notes…"
                        prop:value=notes
                        on:input=move |ev| set_notes.set(event_target_value(&ev))
                    />
                </div>

                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| on_cancel_btn()>"Cancel"</button>
                    <button
                        class="btn-create"
                        on:click=move |_| {
                            let t = title.get();
                            if !t.trim().is_empty() {
                                on_save((t, url.get(), ResourceKind::from_str(&kind.get()), notes.get()));
                            }
                        }
                    >
                        "Add"
                    </button>
                </div>
            </div>
        </div>
    }
}

// ── EditCardModal component ───────────────────────────────────────────────────

#[component]
fn EditCardModal(
    card_id:  ReadSignal<String>,
    columns:  ReadSignal<Vec<Column>>,
    epics:    ReadSignal<Vec<Epic>>,
    stories:  ReadSignal<Vec<Story>>,
    tags:     ReadSignal<Vec<TagDef>>,
    on_save:  impl Fn((String, String, String, String, String, String, Vec<Resource>)) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);

    // Snapshot the card at mount time (modal is re-mounted each open)
    let init = columns.get_untracked()
        .into_iter()
        .flat_map(|c| c.cards.into_iter())
        .find(|c| c.id == card_id.get_untracked());

    let (title, set_title)                   = create_signal(init.as_ref().map(|c| c.title.clone()).unwrap_or_default());
    let (desc, set_desc)                     = create_signal(init.as_ref().map(|c| c.desc.clone()).unwrap_or_default());
    let (tag, set_tag)                       = create_signal(init.as_ref().map(|c| c.tag.clone()).unwrap_or_else(|| "feature".to_string()));
    let (selected_epic, set_selected_epic)   = create_signal(init.as_ref().and_then(|c| c.epic_id.clone()).unwrap_or_default());
    let (selected_story, set_selected_story) = create_signal(init.as_ref().and_then(|c| c.story_id.clone()).unwrap_or_default());
    let (resources, set_resources)           = create_signal(init.map(|c| c.resources.clone()).unwrap_or_default());

    let (link_title, set_link_title) = create_signal(String::new());
    let (link_url, set_link_url)     = create_signal(String::new());

    let on_cancel_overlay = on_cancel.clone();
    let on_cancel_btn     = on_cancel.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| on_cancel_overlay()>
            <div class="modal modal-wide" on:click=|ev| ev.stop_propagation()>
                <h2>"Edit Card"</h2>

                <div class="form-group">
                    <label>"Title"</label>
                    <input
                        type="text"
                        prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Description"</label>
                    <textarea
                        prop:value=desc
                        on:input=move |ev| set_desc.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label>"Tag"</label>
                        <select prop:value=tag on:change=move |ev| set_tag.set(event_target_value(&ev))>
                            { move || tags.get().into_iter().map(|t| {
                                let tid = t.id.clone(); let tn = t.name.clone();
                                view! { <option value=tid>{ tn }</option> }
                            }).collect_view() }
                        </select>
                    </div>
                    <div class="form-group">
                        <label>"Epic"</label>
                        <select
                            prop:value=selected_epic
                            on:change=move |ev| {
                                set_selected_epic.set(event_target_value(&ev));
                                set_selected_story.set(String::new());
                            }
                        >
                            <option value="">"None"</option>
                            { move || epics.get().into_iter().map(|ep| {
                                let eid = ep.id.clone(); let et = ep.title.clone();
                                view! { <option value=eid>{ et }</option> }
                            }).collect_view() }
                        </select>
                    </div>
                    <div class="form-group">
                        <label>"Story"</label>
                        <select prop:value=selected_story on:change=move |ev| set_selected_story.set(event_target_value(&ev))>
                            <option value="">"None"</option>
                            { move || {
                                let cur = selected_epic.get();
                                stories.get().into_iter()
                                    .filter(move |s| s.epic_id == cur)
                                    .map(|s| { let sid = s.id.clone(); let st = s.title.clone(); view! { <option value=sid>{ st }</option> } })
                                    .collect_view()
                            }}
                        </select>
                    </div>
                </div>

                <div class="form-group">
                    <label>"Links"</label>
                    <div class="links-list">
                        { move || resources.get().into_iter().enumerate().map(|(i, res)| {
                            let url   = res.url.clone();
                            let label = if res.title.is_empty() { res.url.clone() } else { res.title.clone() };
                            view! {
                                <div class="link-item">
                                    <a href=url target="_blank" class="link-item-anchor">"🔗 "{ label }</a>
                                    <button class="link-remove-btn" on:click=move |_| {
                                        set_resources.update(|r| { if i < r.len() { r.remove(i); } });
                                    }>"✕"</button>
                                </div>
                            }
                        }).collect_view() }
                    </div>
                    <div class="link-add-row">
                        <input
                            type="text"
                            placeholder="Label (optional)…"
                            prop:value=link_title
                            on:input=move |ev| set_link_title.set(event_target_value(&ev))
                            class="link-input"
                        />
                        <input
                            type="text"
                            placeholder="https://…"
                            prop:value=link_url
                            on:input=move |ev| set_link_url.set(event_target_value(&ev))
                            class="link-input"
                        />
                        <button
                            class="link-add-btn"
                            on:click=move |_| {
                                let u = link_url.get();
                                if !u.trim().is_empty() {
                                    let t = link_title.get();
                                    let label = if t.trim().is_empty() { u.clone() } else { t };
                                    set_resources.update(|r| r.push(Resource::new(&label, &u, ResourceKind::Link, "")));
                                    set_link_title.set(String::new());
                                    set_link_url.set(String::new());
                                }
                            }
                        >"＋ Add"</button>
                    </div>
                </div>

                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| on_cancel_btn()>"Cancel"</button>
                    <button
                        class="btn-create"
                        on:click=move |_| {
                            let t = title.get();
                            if !t.trim().is_empty() {
                                on_save((
                                    card_id.get_untracked(),
                                    t,
                                    desc.get(),
                                    tag.get(),
                                    selected_epic.get(),
                                    selected_story.get(),
                                    resources.get(),
                                ));
                            }
                        }
                    >"Save"</button>
                </div>
            </div>
        </div>
    }
}

// ── AddSpaceModal component ───────────────────────────────────────────────────

#[component]
fn AddSpaceModal(
    on_save: impl Fn(String, String, String) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (name, set_name)   = create_signal(String::new());
    let (color, set_color) = create_signal("#6366f1".to_string());
    let (desc, set_desc)   = create_signal(String::new());

    let oc1 = on_cancel.clone();
    let oc2 = on_cancel.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| oc1()>
            <div class="modal" on:click=|ev| ev.stop_propagation()>
                <h2>"New Space"</h2>
                <div class="form-group">
                    <label>"Name"</label>
                    <input type="text" placeholder="Space name…" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Description"</label>
                    <textarea placeholder="Optional description…" prop:value=desc on:input=move |ev| set_desc.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Color"</label>
                    <select prop:value=color on:change=move |ev| set_color.set(event_target_value(&ev))>
                        <option value="#6366f1">"Indigo"</option>
                        <option value="#f59e0b">"Amber"</option>
                        <option value="#10b981">"Emerald"</option>
                        <option value="#f43f5e">"Rose"</option>
                        <option value="#0ea5e9">"Sky"</option>
                        <option value="#8b5cf6">"Violet"</option>
                        <option value="#64748b">"Slate"</option>
                    </select>
                </div>
                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| oc2()>"Cancel"</button>
                    <button class="btn-create" on:click=move |_| {
                        let n = name.get();
                        if !n.trim().is_empty() { on_save(n, color.get(), desc.get()); }
                    }>"Create"</button>
                </div>
            </div>
        </div>
    }
}

// ── AddTagModal component ─────────────────────────────────────────────────────

#[component]
fn AddTagModal(
    on_save: impl Fn(String, String) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (name, set_name)   = create_signal(String::new());
    let (color, set_color) = create_signal("#6366f1".to_string());

    let oc1 = on_cancel.clone();
    let oc2 = on_cancel.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| oc1()>
            <div class="modal" on:click=|ev| ev.stop_propagation()>
                <h2>"New Tag"</h2>
                <div class="form-group">
                    <label>"Name"</label>
                    <input type="text" placeholder="Tag name…" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Color"</label>
                    <input type="color" prop:value=color on:input=move |ev| set_color.set(event_target_value(&ev))
                           style="width:100%;height:40px;border-radius:8px;border:1px solid #334155;background:#0f172a;cursor:pointer;" />
                </div>
                <div class="form-group">
                    <label>"Preview"</label>
                    { move || {
                        let n  = name.get();
                        let c  = color.get();
                        let st = format!("background:{}28;color:{};border:1px solid {}55;padding:2px 8px;border-radius:5px;font-size:0.8rem;font-weight:600;display:inline-block;", c, c, c);
                        let nm = if n.is_empty() { "Tag name".to_string() } else { n };
                        view! { <span style=st>{ nm }</span> }
                    }}
                </div>
                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| oc2()>"Cancel"</button>
                    <button class="btn-create" on:click=move |_| {
                        let n = name.get();
                        if !n.trim().is_empty() { on_save(n, color.get()); }
                    }>"Create"</button>
                </div>
            </div>
        </div>
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    mount_to_body(App);
}
