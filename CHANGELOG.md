# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.3.0] — 2026-03-30

### Added
- **Epics** — data model with `id`, `title`, `desc`, `color`, `deadline`, `estimated_hours`, `logged_secs`
- **Stories** — grouped under Epics; cards can be linked to a Story
- **Subtasks** — `Vec<Subtask>` on each card with `done` toggle; progress shown on card face
- **Dashboard view** — tab-switched Epic grid with: color bar, deadline badge (red if overdue), progress bar, logged vs estimated time, stories list
- **View tabs** — "Board" / "Dashboard" pill nav in header
- **Timer → Epic integration** — epic selector in Pomodoro widget; every Work tick increments selected epic's `logged_secs`
- **Add Epic modal** — title, description, colour picker (6 presets), date picker, estimated hours
- **Add Story modal** — epic selector + story title
- **Card → Epic/Story linking** — Add Card modal extended with Epic and Story dropdowns
- **Epic colour stripe on cards** — thin left border using epic's hex colour
- **MCP epic tools** — `list_epics`, `add_epic`, `delete_epic`, `log_time`, `get_epic_summary`
- **MCP story tools** — `list_stories`, `add_story`, `delete_story`
- **MCP state migration** — `board_state.json` auto-upgraded to include `epics`/`stories` keys
- `Card::new_full()` constructor accepting `epic_id` and `story_id`

### Changed
- `get_board_summary` now includes Epic column in card tables
- `add_card` MCP tool accepts optional `epic_id` and `story_id` params

---

## [0.2.0] — 2026-03-29

### Added
- **Dynamic columns** — `+ Add column` ghost button opens modal to create custom columns at runtime
- **Pomodoro timer** — 25 min Work / 5 min Break cycles with ▶/⏸, ↺ reset, ⏭ skip; stored in `StoredValue<Option<Interval>>`
- **MCP server** (`mcp_server/server.py`) — Python FastMCP server with 8 tools: `list_columns`, `list_cards`, `add_card`, `move_card`, `delete_card`, `add_column`, `delete_column`, `get_board_summary`
- Board state persisted to `board_state.json`; auto-created on first run
- `AddColumnModal` component
- `AddCardModal` column picker now reflects dynamically created columns

### Changed
- `Column.id` and `Column.title` changed from `&'static str` to `String` to support runtime creation
- `AddCardModal` accepts `columns: ReadSignal<Vec<Column>>` prop instead of a hardcoded list

---

## [0.1.0] — 2026-03-29

### Added
- Initial Rust + Leptos 0.6 WASM Kanban board
- Four default columns: To Do, In Progress, Review, Done
- Cards with title, description, tag (Feature / Bug / Task / Design)
- HTML5 drag & drop between columns with visual drop-zone highlight
- Delete card button
- Add card modal with column, tag, and description fields
- Dark slate theme (no CSS framework)
- Trunk dev server with hot-reload (`trunk serve`)
- Production build with `wasm-opt` (`trunk build --release`)
- `Trunk.toml` configuration
