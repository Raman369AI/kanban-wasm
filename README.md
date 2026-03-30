# Kanban Board — Rust + WASM

A fully client-side Kanban board that compiles to **WebAssembly** using [Leptos](https://leptos.dev/) and [Trunk](https://trunkrs.dev/). Ships with a **Python MCP server** so AI agents can read and write the board programmatically.

---

## Features

### Board
- **Columns** — default: To Do / In Progress / Review / Done; add unlimited custom columns
- **Cards (Tasks)** — drag & drop between columns, tag badges (Feature / Bug / Task / Design), delete
- **Epics** — color-coded, deadline, estimated hours, time tracking
- **Stories** — grouped under Epics; link cards to a Story
- **Subtasks** — inline checklist on each card with progress indicator
- **Add Card modal** — pick column, epic, story, tag

### Dashboard
- Epic grid with color bar, deadline (highlighted red if overdue)
- Progress bar: done-column tasks / total tasks per epic
- Time tracker: logged hours vs estimated, fed by the Pomodoro timer

### Pomodoro Timer
- 25 min Work / 5 min Break cycles
- **Epic selector** — every Work-mode tick adds 1 s to the selected epic's logged time
- ▶ / ⏸ toggle, ↺ reset, ⏭ skip to next mode

### MCP Server (16 tools)
AI agents (Claude, Cursor, etc.) can interact with the board via the Model Context Protocol:

| Category | Tools |
|----------|-------|
| Columns | `list_columns`, `add_column`, `delete_column` |
| Cards | `list_cards`, `add_card`, `move_card`, `delete_card` |
| Epics | `list_epics`, `add_epic`, `delete_epic`, `log_time`, `get_epic_summary` |
| Stories | `list_stories`, `add_story`, `delete_story` |
| Board | `get_board_summary` |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2021) |
| UI framework | [Leptos 0.6](https://leptos.dev/) (fine-grained reactivity) |
| WASM bundler | [Trunk 0.21](https://trunkrs.dev/) |
| Timer | [gloo-timers 0.3](https://docs.rs/gloo-timers) |
| IDs | [uuid 1 (v4)](https://docs.rs/uuid) |
| MCP server | Python 3.12 + [mcp\[cli\]](https://pypi.org/project/mcp/) |

---

## Getting Started

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# Trunk
cargo install trunk

# Python ≥ 3.11 (for MCP server)
```

### Run the board

```bash
# Dev server with hot-reload at http://localhost:8080
trunk serve

# Production build → dist/
trunk build --release
```

### Run the MCP server

```bash
cd mcp_server
python -m venv .venv && .venv/bin/pip install -r requirements.txt
.venv/bin/python server.py
```

#### Register with Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "kanban": {
      "command": "/absolute/path/to/mcp_server/.venv/bin/python",
      "args": ["/absolute/path/to/mcp_server/server.py"]
    }
  }
}
```

---

## Project Structure

```
kanban-wasm/
├── src/
│   └── main.rs          # Full Leptos app (~1100 lines)
├── mcp_server/
│   ├── server.py        # Python MCP server (16 tools)
│   └── requirements.txt
├── style.css            # Dark-theme CSS (no framework)
├── index.html           # Trunk entry point
├── Cargo.toml
├── Trunk.toml
└── CHANGELOG.md
```

---

## MCP Server State

Board state is persisted to `mcp_server/board_state.json` after every mutation. The file is auto-created with sample data on first run and auto-migrated when new fields are added.

---

## License

MIT
