use leptos::*;
use uuid::Uuid;
use std::rc::Rc;
use gloo_timers::callback::Interval;

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum Tag {
    Feature,
    Bug,
    Task,
    Design,
}

impl Tag {
    fn label(&self) -> &'static str {
        match self {
            Tag::Feature => "Feature",
            Tag::Bug     => "Bug",
            Tag::Task    => "Task",
            Tag::Design  => "Design",
        }
    }
    fn css_class(&self) -> &'static str {
        match self {
            Tag::Feature => "tag-feature",
            Tag::Bug     => "tag-bug",
            Tag::Task    => "tag-task",
            Tag::Design  => "tag-design",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "Bug"    => Tag::Bug,
            "Task"   => Tag::Task,
            "Design" => Tag::Design,
            _        => Tag::Feature,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Subtask {
    pub id:    String,
    pub title: String,
    pub done:  bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Story {
    pub id:      String,
    pub epic_id: String,
    pub title:   String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Epic {
    pub id:               String,
    pub title:            String,
    pub desc:             String,
    pub color:            String,
    pub deadline:         String,
    pub estimated_hours:  f32,
    pub logged_secs:      u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub id:       String,
    pub title:    String,
    pub desc:     String,
    pub tag:      Tag,
    pub epic_id:  Option<String>,
    pub story_id: Option<String>,
    pub subtasks: Vec<Subtask>,
}

impl Card {
    fn new(title: &str, desc: &str, tag: Tag) -> Self {
        Card {
            id:       Uuid::new_v4().to_string(),
            title:    title.to_string(),
            desc:     desc.to_string(),
            tag,
            epic_id:  None,
            story_id: None,
            subtasks: vec![],
        }
    }

    fn new_full(title: &str, desc: &str, tag: Tag, epic_id: Option<String>, story_id: Option<String>) -> Self {
        Card {
            id:       Uuid::new_v4().to_string(),
            title:    title.to_string(),
            desc:     desc.to_string(),
            tag,
            epic_id,
            story_id,
            subtasks: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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
                Card::new("Design system tokens",  "Define color / spacing tokens for the design system.", Tag::Design),
                Card::new("Auth flow",              "Implement OAuth2 login + refresh token rotation.",    Tag::Feature),
                Card::new("Fix nav overflow bug",  "Mobile nav overflows viewport at <375px.",            Tag::Bug),
            ],
        },
        Column {
            id:    "in-progress".to_string(),
            title: "In Progress".to_string(),
            cards: vec![
                Card::new("Kanban WASM POC", "Rust + Leptos kanban that compiles to WebAssembly.", Tag::Task),
            ],
        },
        Column {
            id:    "review".to_string(),
            title: "Review".to_string(),
            cards: vec![
                Card::new("API rate limiting", "Add sliding-window rate limiter to REST endpoints.", Tag::Feature),
            ],
        },
        Column {
            id:    "done".to_string(),
            title: "Done".to_string(),
            cards: vec![
                Card::new("Repo setup", "Init monorepo, CI pipeline, branch protection rules.", Tag::Task),
            ],
        },
    ]
}

fn initial_epics() -> Vec<Epic> {
    vec![
        Epic {
            id:              "epic-1".to_string(),
            title:           "Platform v2".to_string(),
            desc:            "Full platform rewrite".to_string(),
            color:           "#6366f1".to_string(),
            deadline:        "2026-06-30".to_string(),
            estimated_hours: 120.0,
            logged_secs:     0,
        },
        Epic {
            id:              "epic-2".to_string(),
            title:           "Design System".to_string(),
            desc:            "Component library and tokens".to_string(),
            color:           "#f59e0b".to_string(),
            deadline:        "2026-04-15".to_string(),
            estimated_hours: 40.0,
            logged_secs:     0,
        },
    ]
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    let (columns, set_columns)               = create_signal(initial_columns());
    let (dragging, set_dragging)             = create_signal::<Option<(String, String)>>(None);
    let (show_modal, set_show_modal)         = create_signal(false);
    let (modal_col, set_modal_col)           = create_signal("todo".to_string());
    let (show_col_modal, set_show_col_modal) = create_signal(false);

    let (epics, set_epics)                   = create_signal(initial_epics());
    let (stories, set_stories)               = create_signal(Vec::<Story>::new());
    let (active_view, set_active_view)       = create_signal("board");
    let (timer_epic_id, set_timer_epic_id)   = create_signal(Option::<String>::None);
    let (show_epic_modal, set_show_epic_modal)   = create_signal(false);
    let (show_story_modal, set_show_story_modal) = create_signal(false);
    let (story_epic_id, set_story_epic_id)       = create_signal(String::new());

    // ── Pomodoro timer signals ─────────────────────────────────────────────────
    let (timer_secs, set_timer_secs)       = create_signal(25u32 * 60);
    let (timer_running, set_timer_running) = create_signal(false);
    let (timer_mode, set_timer_mode)       = create_signal("Work");

    let interval_handle: StoredValue<Option<Interval>> = store_value(None::<Interval>);

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

    // ── add card handler ──────────────────────────────────────────────────────

    let on_add_card = move |col_id: String, title: String, desc: String, tag: String, epic_id: Option<String>, story_id: Option<String>| {
        set_columns.update(|cols| {
            if let Some(col) = cols.iter_mut().find(|c| c.id == col_id) {
                col.cards.push(Card::new_full(&title, &desc, Tag::from_str(&tag), epic_id, story_id));
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

    // ── epic & story handlers ─────────────────────────────────────────────────

    let on_add_epic = move |title: String, desc: String, color: String, deadline: String, est: f32| {
        set_epics.update(|epics| {
            epics.push(Epic {
                id:              Uuid::new_v4().to_string(),
                title,
                desc,
                color,
                deadline,
                estimated_hours: est,
                logged_secs:     0,
            });
        });
    };

    let on_add_story = move |epic_id: String, title: String| {
        set_stories.update(|stories| {
            stories.push(Story {
                id:      Uuid::new_v4().to_string(),
                epic_id,
                title,
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
                            if let Some(eid) = timer_epic_id.get() {
                                set_epics.update(|epics| {
                                    if let Some(ep) = epics.iter_mut().find(|e| e.id == eid) {
                                        ep.logged_secs += 1;
                                    }
                                });
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
                    <button
                        class=move || if active_view.get() == "board" { "tab active" } else { "tab" }
                        on:click=move |_| set_active_view.set("board")
                    >
                        "Board"
                    </button>
                    <button
                        class=move || if active_view.get() == "dashboard" { "tab active" } else { "tab" }
                        on:click=move |_| set_active_view.set("dashboard")
                    >
                        "Dashboard"
                    </button>
                </nav>

                // ── Pomodoro Timer ───────────────────────────────────────────
                <div class="timer">
                    <span class="timer-mode">{ move || timer_mode.get() }</span>
                    <select
                        class="timer-epic-select"
                        prop:value=move || timer_epic_id.get().unwrap_or_default()
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            set_timer_epic_id.set(if v.is_empty() { None } else { Some(v) });
                        }
                    >
                        <option value="">"-- no epic --"</option>
                        { move || epics.get().into_iter().map(|ep| {
                            let eid    = ep.id.clone();
                            let etitle = ep.title.clone();
                            view! { <option value=eid>{ etitle }</option> }
                        }).collect_view() }
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
                            let col_id          = col.id.clone();
                            let col_id_drop     = col_id.clone();
                            let col_id_add      = col_id.clone();
                            let col_id_modal    = col_id.clone();
                            let col_id_for_card = col_id.clone();
                            let on_drop_c       = on_drop.clone();
                            let on_delete_c     = on_delete.clone();
                            let on_drag_start_c = on_drag_start.clone();

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
                                    class=move || if drag_over.get() { "column drag-over" } else { "column" }
                                    on:dragover=move |ev| { ev.prevent_default(); set_drag_over.set(true); }
                                    on:dragleave=move |_| { set_drag_over.set(false); }
                                    on:drop=move |ev| {
                                        ev.prevent_default();
                                        set_drag_over.set(false);
                                        on_drop_c(col_id_drop.clone());
                                    }
                                >
                                    <div class="column-header">
                                        <span class=title_class>{ col.title.clone() }</span>
                                        <span class="column-count">
                                            { move || {
                                                columns.get()
                                                    .iter()
                                                    .find(|c| c.id == col_id)
                                                    .map(|c| c.cards.len())
                                                    .unwrap_or(0)
                                            }}
                                        </span>
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
                                                let src_col            = col_id_for_card.clone();
                                                let on_drag_start_card = on_drag_start_c.clone();
                                                let on_delete_card     = on_delete_c.clone();
                                                let tag_class          = card.tag.css_class();
                                                let tag_label          = card.tag.label();
                                                let card_epic_id       = card.epic_id.clone();
                                                let subtask_done       = card.subtasks.iter().filter(|s| s.done).count();
                                                let subtask_total      = card.subtasks.len();

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
                                                        <div class="card-footer">
                                                            <span class=format!("card-tag {}", tag_class)>
                                                                { tag_label }
                                                            </span>
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
                        <h2>"Epics"</h2>
                        <button
                            class="add-card-btn"
                            on:click=move |_| set_show_epic_modal.set(true)
                        >
                            "+ New Epic"
                        </button>
                    </div>
                    <div class="epic-grid">
                        <For
                            each=move || epics.get()
                            key=|e| e.id.clone()
                            children=move |epic| {
                                let epic_id_prog_p  = epic.id.clone();
                                let epic_id_prog_l  = epic.id.clone();
                                let epic_id_stories = epic.id.clone();
                                let epic_id_story   = epic.id.clone();
                                let epic_color      = epic.color.clone();
                                let epic_title      = epic.title.clone();
                                let epic_desc       = epic.desc.clone();
                                let epic_deadline   = epic.deadline.clone();
                                let est_h           = epic.estimated_hours;
                                let logged_secs     = epic.logged_secs;
                                let logged_h        = logged_secs / 3600;
                                let logged_m        = (logged_secs % 3600) / 60;
                                let overdue_class   = if epic.deadline.as_str() < "2026-03-30" { "epic-deadline overdue" } else { "epic-deadline" };
                                let color_bar_style = format!("background:{};", epic_color);

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
                                                <span class=overdue_class>{ epic_deadline }</span>
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
                                                <span class="time-label">"Logged"</span>
                                                <span class="time-value">{ format!("{}h {}m", logged_h, logged_m) }</span>
                                                <span class="time-sep">"/"</span>
                                                <span class="time-est">{ format!("{}h est.", est_h) }</span>
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
                                                        view! {
                                                            <div class="story-chip">{ story.title.clone() }</div>
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
                                        </div>
                                    </div>
                                }
                            }
                        />
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
                    on_save=move |title, desc, color, deadline, est| {
                        on_add_epic(title, desc, color, deadline, est);
                        set_show_epic_modal.set(false);
                    }
                    on_cancel=move || set_show_epic_modal.set(false)
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
    on_save: impl Fn((String, String, String, String, String, String)) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (title, set_title)                     = create_signal(String::new());
    let (desc, set_desc)                       = create_signal(String::new());
    let (tag, set_tag)                         = create_signal("Feature".to_string());
    let (selected_col, set_selected_col)       = create_signal(col_id.get_untracked());
    let (selected_epic, set_selected_epic)     = create_signal(String::new());
    let (selected_story, set_selected_story)   = create_signal(String::new());

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
                    <select
                        prop:value=tag
                        on:change=move |ev| set_tag.set(event_target_value(&ev))
                    >
                        <option value="Feature">"Feature"</option>
                        <option value="Bug">"Bug"</option>
                        <option value="Task">"Task"</option>
                        <option value="Design">"Design"</option>
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
    on_save: impl Fn(String, String, String, String, f32) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let on_cancel = Rc::new(on_cancel);
    let (title, set_title)           = create_signal(String::new());
    let (desc, set_desc)             = create_signal(String::new());
    let (color, set_color)           = create_signal("#6366f1".to_string());
    let (deadline, set_deadline)     = create_signal(String::new());
    let (est_hours, set_est_hours)   = create_signal(8.0f32);

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

                <div class="modal-actions">
                    <button class="btn-cancel" on:click=move |_| on_cancel_btn()>"Cancel"</button>
                    <button
                        class="btn-create"
                        on:click=move |_| {
                            let t = title.get();
                            if !t.trim().is_empty() {
                                on_save(t, desc.get(), color.get(), deadline.get(), est_hours.get());
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

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    mount_to_body(App);
}
