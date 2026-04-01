import json
import uuid
from pathlib import Path
from mcp.server.fastmcp import FastMCP

mcp = FastMCP("Kanban Board")

STATE_FILE = Path(__file__).parent / "board_state.json"

# ── State helpers ─────────────────────────────────────────────────────────────

def _default_state() -> dict:
    def card(title: str, desc: str, tag: str, epic_id: str | None = None) -> dict:
        return {"id": str(uuid.uuid4()), "title": title, "desc": desc, "tag": tag,
                "epic_id": epic_id, "story_id": None, "subtasks": [],
                "logged_secs": 0, "resources": []}

    return {
        "columns": [
            {
                "id": "todo",
                "title": "To Do",
                "cards": [
                    card("Design system tokens", "Define color / spacing tokens for the design system.", "Design", "epic-2"),
                    card("Auth flow", "Implement OAuth2 login + refresh token rotation.", "Feature", "epic-1"),
                    card("Fix nav overflow bug", "Mobile nav overflows viewport at <375px.", "Bug"),
                ],
            },
            {
                "id": "in-progress",
                "title": "In Progress",
                "cards": [
                    card("Kanban WASM POC", "Rust + Leptos kanban that compiles to WebAssembly.", "Task", "epic-1"),
                ],
            },
            {
                "id": "review",
                "title": "Review",
                "cards": [
                    card("API rate limiting", "Add sliding-window rate limiter to REST endpoints.", "Feature", "epic-1"),
                ],
            },
            {
                "id": "done",
                "title": "Done",
                "cards": [
                    card("Repo setup", "Init monorepo, CI pipeline, branch protection rules.", "Task"),
                ],
            },
        ],
        "epics": [
            {
                "id": "epic-1",
                "title": "Platform v2",
                "desc": "Full platform rewrite",
                "color": "#6366f1",
                "deadline": "2026-06-30",
                "estimated_hours": 120.0,
                "direct_logged_secs": 0,
                "resources": [],
            },
            {
                "id": "epic-2",
                "title": "Design System",
                "desc": "Component library and tokens",
                "color": "#f59e0b",
                "deadline": "2026-04-15",
                "estimated_hours": 40.0,
                "direct_logged_secs": 0,
                "resources": [],
            },
        ],
        "stories": [],
    }


def _load() -> dict:
    if STATE_FILE.exists():
        try:
            state = json.loads(STATE_FILE.read_text())
            # Migrate older state files that predate epics/stories
            if "epics" not in state:
                defaults = _default_state()
                state["epics"] = defaults["epics"]
                state["stories"] = defaults["stories"]
                _save(state)
            if "stories" not in state:
                state["stories"] = []
                _save(state)
            # Migrate: resources fields
            for col in state.get("columns", []):
                for card in col.get("cards", []):
                    if "resources" not in card:
                        card["resources"] = []
                    if "logged_secs" not in card:
                        card["logged_secs"] = 0
            for s in state.get("stories", []):
                if "resources" not in s:
                    s["resources"] = []
                if "direct_logged_secs" not in s:
                    s["direct_logged_secs"] = 0
            for ep in state.get("epics", []):
                if "resources" not in ep:
                    ep["resources"] = []
                # Migrate logged_secs -> direct_logged_secs
                if "direct_logged_secs" not in ep:
                    ep["direct_logged_secs"] = ep.pop("logged_secs", 0)
                elif "logged_secs" in ep:
                    # Both present — keep direct_logged_secs, remove legacy key
                    ep.pop("logged_secs", None)
            _save(state)
            return state
        except (json.JSONDecodeError, OSError):
            pass
    state = _default_state()
    _save(state)
    return state


def _save(state: dict) -> None:
    STATE_FILE.write_text(json.dumps(state, indent=2))


# Load once at startup; tools mutate this in-process and persist to disk.
_state = _load()


# ── Lookup helpers ────────────────────────────────────────────────────────────

def _find_column(state: dict, column_id: str) -> dict | None:
    return next((c for c in state["columns"] if c["id"] == column_id), None)


def _find_card(state: dict, card_id: str) -> tuple[dict | None, dict | None]:
    for col in state["columns"]:
        for card in col["cards"]:
            if card["id"] == card_id:
                return col, card
    return None, None


def _find_epic(state: dict, epic_id: str) -> dict | None:
    return next((e for e in state["epics"] if e["id"] == epic_id), None)


def _find_story(state: dict, story_id: str) -> dict | None:
    return next((s for s in state["stories"] if s["id"] == story_id), None)


# ── Aggregation helpers ───────────────────────────────────────────────────────

def _story_total_secs(story_id: str) -> int:
    direct = next((s["direct_logged_secs"] for s in _state["stories"] if s["id"] == story_id), 0)
    task_secs = sum(
        c.get("logged_secs", 0)
        for col in _state["columns"]
        for c in col["cards"]
        if c.get("story_id") == story_id
    )
    return direct + task_secs


def _epic_total_secs(epic_id: str) -> int:
    ep = _find_epic(_state, epic_id)
    direct = ep.get("direct_logged_secs", 0) if ep else 0
    story_secs = sum(
        _story_total_secs(s["id"])
        for s in _state["stories"]
        if s["epic_id"] == epic_id
    )
    # Cards on the epic but not assigned to any story
    orphan_secs = sum(
        c.get("logged_secs", 0)
        for col in _state["columns"]
        for c in col["cards"]
        if c.get("epic_id") == epic_id and not c.get("story_id")
    )
    return direct + story_secs + orphan_secs


# ── Column / Card Tools ───────────────────────────────────────────────────────

@mcp.tool()
def list_columns() -> list[dict]:
    """List all columns on the board with their card counts."""
    return [
        {"id": col["id"], "title": col["title"], "card_count": len(col["cards"])}
        for col in _state["columns"]
    ]


@mcp.tool()
def list_cards(column_id: str) -> list[dict]:
    """List cards in a specific column, or all cards across the board.

    Args:
        column_id: Column id to filter by, or "all" for every card.
    """
    results = []
    for col in _state["columns"]:
        if column_id != "all" and col["id"] != column_id:
            continue
        for card in col["cards"]:
            results.append({
                "id": card["id"],
                "title": card["title"],
                "desc": card["desc"],
                "tag": card["tag"],
                "epic_id": card.get("epic_id"),
                "story_id": card.get("story_id"),
                "subtask_count": len(card.get("subtasks", [])),
                "logged_secs": card.get("logged_secs", 0),
                "resource_count": len(card.get("resources", [])),
                "column_id": col["id"],
                "column_title": col["title"],
            })
    return results


@mcp.tool()
def add_card(column_id: str, title: str, desc: str = "", tag: str = "Task",
             epic_id: str = "", story_id: str = "") -> dict:
    """Add a new card to the specified column.

    Args:
        column_id: Column to add the card to.
        title: Card title.
        desc: Optional description.
        tag: Feature | Bug | Task | Design (default: Task).
        epic_id: Optional epic id to link this card to.
        story_id: Optional story id to link this card to.
    """
    col = _find_column(_state, column_id)
    if col is None:
        raise ValueError(f"Column '{column_id}' not found.")
    new_card = {
        "id": str(uuid.uuid4()),
        "title": title,
        "desc": desc,
        "tag": tag,
        "epic_id": epic_id or None,
        "story_id": story_id or None,
        "subtasks": [],
        "logged_secs": 0,
        "resources": [],
    }
    col["cards"].append(new_card)
    _save(_state)
    return {"id": new_card["id"]}


@mcp.tool()
def move_card(card_id: str, target_column_id: str) -> dict:
    """Move a card to a different column.

    Args:
        card_id: Card to move.
        target_column_id: Destination column id.
    """
    src_col, card = _find_card(_state, card_id)
    if card is None:
        raise ValueError(f"Card '{card_id}' not found.")
    target_col = _find_column(_state, target_column_id)
    if target_col is None:
        raise ValueError(f"Column '{target_column_id}' not found.")
    src_col["cards"].remove(card)
    target_col["cards"].append(card)
    _save(_state)
    return {"card_id": card_id, "target_column_id": target_column_id}


@mcp.tool()
def delete_card(card_id: str) -> dict:
    """Delete a card from the board.

    Args:
        card_id: Card to delete.
    """
    col, card = _find_card(_state, card_id)
    if card is None:
        raise ValueError(f"Card '{card_id}' not found.")
    col["cards"].remove(card)
    _save(_state)
    return {"deleted_card_id": card_id}


@mcp.tool()
def add_column(title: str) -> dict:
    """Create a new column on the board.

    Args:
        title: Display title for the new column.
    """
    new_id = str(uuid.uuid4())
    _state["columns"].append({"id": new_id, "title": title, "cards": []})
    _save(_state)
    return {"id": new_id}


@mcp.tool()
def delete_column(column_id: str) -> dict:
    """Delete a column. Fails if it still has cards.

    Args:
        column_id: Column to delete.
    """
    col = _find_column(_state, column_id)
    if col is None:
        raise ValueError(f"Column '{column_id}' not found.")
    if col["cards"]:
        raise ValueError(
            f"Column still has {len(col['cards'])} card(s). Move or delete them first."
        )
    _state["columns"].remove(col)
    _save(_state)
    return {"deleted_column_id": column_id}


# ── Epic Tools ────────────────────────────────────────────────────────────────

@mcp.tool()
def list_epics() -> list[dict]:
    """List all epics with time tracking info."""
    result = []
    for ep in _state["epics"]:
        direct = ep.get("direct_logged_secs", 0)
        total  = _epic_total_secs(ep["id"])
        result.append({
            "id": ep["id"],
            "title": ep["title"],
            "color": ep["color"],
            "deadline": ep["deadline"],
            "estimated_hours": ep["estimated_hours"],
            "direct_logged_secs": direct,
            "total_logged_secs": total,
            "total_logged_hours": round(total / 3600, 2),
        })
    return result


@mcp.tool()
def add_epic(title: str, desc: str = "", color: str = "#6366f1",
             deadline: str = "", estimated_hours: float = 8.0) -> dict:
    """Create a new epic.

    Args:
        title: Epic title.
        desc: Optional description.
        color: Hex color e.g. #6366f1, #f59e0b, #10b981, #f43f5e, #0ea5e9, #8b5cf6.
        deadline: ISO date string YYYY-MM-DD.
        estimated_hours: Estimated effort in hours (default 8).
    """
    new_id = str(uuid.uuid4())
    _state["epics"].append({
        "id": new_id,
        "title": title,
        "desc": desc,
        "color": color,
        "deadline": deadline,
        "estimated_hours": estimated_hours,
        "direct_logged_secs": 0,
        "resources": [],
    })
    _save(_state)
    return {"id": new_id}


@mcp.tool()
def delete_epic(epic_id: str) -> dict:
    """Delete an epic. Fails if stories are linked to it.

    Args:
        epic_id: Epic to delete.
    """
    ep = _find_epic(_state, epic_id)
    if ep is None:
        raise ValueError(f"Epic '{epic_id}' not found.")
    linked_stories = [s for s in _state["stories"] if s["epic_id"] == epic_id]
    if linked_stories:
        raise ValueError(
            f"Epic has {len(linked_stories)} story/stories. Delete them first."
        )
    _state["epics"].remove(ep)
    _save(_state)
    return {"deleted_epic_id": epic_id}


@mcp.tool()
def log_time(entity_id: str, seconds: int, entity_type: str = "epic") -> dict:
    """Add logged time to an epic, story, or card.

    Args:
        entity_id: Id of the entity to log time against.
        seconds: Number of seconds to add.
        entity_type: "epic" | "story" | "card" (default: "epic").
    """
    added = max(0, seconds)
    if entity_type == "epic":
        ep = _find_epic(_state, entity_id)
        if ep is None:
            raise ValueError(f"Epic '{entity_id}' not found.")
        ep["direct_logged_secs"] = ep.get("direct_logged_secs", 0) + added
        _save(_state)
        total = _epic_total_secs(entity_id)
        return {
            "entity_type": "epic",
            "entity_id": entity_id,
            "direct_logged_secs": ep["direct_logged_secs"],
            "total_logged_secs": total,
            "total_logged_hours": round(total / 3600, 2),
        }
    elif entity_type == "story":
        s = _find_story(_state, entity_id)
        if s is None:
            raise ValueError(f"Story '{entity_id}' not found.")
        s["direct_logged_secs"] = s.get("direct_logged_secs", 0) + added
        _save(_state)
        total = _story_total_secs(entity_id)
        return {
            "entity_type": "story",
            "entity_id": entity_id,
            "direct_logged_secs": s["direct_logged_secs"],
            "total_logged_secs": total,
        }
    elif entity_type == "card":
        _, card = _find_card(_state, entity_id)
        if card is None:
            raise ValueError(f"Card '{entity_id}' not found.")
        card["logged_secs"] = card.get("logged_secs", 0) + added
        _save(_state)
        return {
            "entity_type": "card",
            "entity_id": entity_id,
            "logged_secs": card["logged_secs"],
        }
    else:
        raise ValueError(f"Unknown entity_type '{entity_type}'. Use epic, story, or card.")


@mcp.tool()
def get_epic_summary(epic_id: str) -> str:
    """Return a rich Markdown summary for one epic including progress and time.

    Args:
        epic_id: Epic to summarise.
    """
    ep = _find_epic(_state, epic_id)
    if ep is None:
        raise ValueError(f"Epic '{epic_id}' not found.")

    # Progress: cards linked to this epic
    all_cards = [c for col in _state["columns"] for c in col["cards"]]
    epic_cards = [c for c in all_cards if c.get("epic_id") == epic_id]
    done_col = _find_column(_state, "done")
    done_cards = [c for c in (done_col["cards"] if done_col else []) if c.get("epic_id") == epic_id]
    total = len(epic_cards)
    done = len(done_cards)
    pct = int(done * 100 / total) if total else 0

    total_secs  = _epic_total_secs(epic_id)
    direct_secs = ep.get("direct_logged_secs", 0)
    total_h  = total_secs // 3600
    total_m  = (total_secs % 3600) // 60
    direct_h = direct_secs // 3600
    direct_m = (direct_secs % 3600) // 60

    stories = [s for s in _state["stories"] if s["epic_id"] == epic_id]

    lines = [
        f"# {ep['title']}",
        f"> {ep['desc']}" if ep.get("desc") else "",
        "",
        f"**Color:** {ep['color']}  **Deadline:** {ep['deadline'] or 'none'}",
        "",
        f"## Progress",
        f"`{done}/{total} tasks done` — **{pct}%**",
        "",
        f"## Time",
        f"- Total logged: **{total_h}h {total_m}m**",
        f"- Direct logged: **{direct_h}h {direct_m}m**",
        f"- Estimated: **{ep['estimated_hours']}h**",
        "",
    ]

    if stories:
        lines += ["## Stories", ""]
        for s in stories:
            story_cards = [c for c in epic_cards if c.get("story_id") == s["id"]]
            story_total  = _story_total_secs(s["id"])
            story_h = story_total // 3600
            story_m = (story_total % 3600) // 60
            lines.append(f"- **{s['title']}** ({len(story_cards)} tasks, {story_h}h {story_m}m logged)")
        lines.append("")

    if epic_cards:
        lines += ["## Tasks", "| Title | Tag | Column | Logged |", "|-------|-----|--------|--------|"]
        col_map = {col["id"]: col["title"] for col in _state["columns"]}
        for c in all_cards:
            if c.get("epic_id") == epic_id:
                col_name = next(
                    (col["title"] for col in _state["columns"] if any(x["id"] == c["id"] for x in col["cards"])),
                    "?"
                )
                c_secs = c.get("logged_secs", 0)
                c_h = c_secs // 3600
                c_m = (c_secs % 3600) // 60
                lines.append(f"| {c['title']} | {c['tag']} | {col_name} | {c_h}h {c_m}m |")

    return "\n".join(l for l in lines)


# ── Story Tools ───────────────────────────────────────────────────────────────

@mcp.tool()
def list_stories(epic_id: str = "all") -> list[dict]:
    """List stories, optionally filtered by epic.

    Args:
        epic_id: Epic id to filter by, or "all" (default).
    """
    stories = _state["stories"]
    if epic_id != "all":
        stories = [s for s in stories if s["epic_id"] == epic_id]
    return [
        {
            "id": s["id"],
            "epic_id": s["epic_id"],
            "title": s["title"],
            "direct_logged_secs": s.get("direct_logged_secs", 0),
            "total_logged_secs": _story_total_secs(s["id"]),
        }
        for s in stories
    ]


@mcp.tool()
def add_story(epic_id: str, title: str) -> dict:
    """Add a story under an epic.

    Args:
        epic_id: Parent epic id.
        title: Story title.
    """
    ep = _find_epic(_state, epic_id)
    if ep is None:
        raise ValueError(f"Epic '{epic_id}' not found.")
    new_id = str(uuid.uuid4())
    _state["stories"].append({
        "id": new_id,
        "epic_id": epic_id,
        "title": title,
        "direct_logged_secs": 0,
        "resources": [],
    })
    _save(_state)
    return {"id": new_id}


@mcp.tool()
def delete_story(story_id: str) -> dict:
    """Delete a story.

    Args:
        story_id: Story to delete.
    """
    story = _find_story(_state, story_id)
    if story is None:
        raise ValueError(f"Story '{story_id}' not found.")
    _state["stories"].remove(story)
    _save(_state)
    return {"deleted_story_id": story_id}


# ── Resource Tools ────────────────────────────────────────────────────────────

@mcp.tool()
def add_resource(entity_type: str, entity_id: str, title: str, url: str = "", kind: str = "Link", notes: str = "") -> dict:
    """Add a resource (link/note/doc) to any entity on the board.

    Args:
        entity_type: "epic", "story", or "card"
        entity_id: id of the entity
        title: resource title
        url: URL or reference string
        kind: Link | Note | Doc
        notes: optional notes
    """
    new_res = {"id": str(uuid.uuid4()), "title": title, "url": url, "kind": kind, "notes": notes}
    if entity_type == "epic":
        ep = _find_epic(_state, entity_id)
        if ep is None:
            raise ValueError(f"Epic '{entity_id}' not found.")
        ep.setdefault("resources", []).append(new_res)
    elif entity_type == "story":
        s = _find_story(_state, entity_id)
        if s is None:
            raise ValueError(f"Story '{entity_id}' not found.")
        s.setdefault("resources", []).append(new_res)
    elif entity_type == "card":
        _, card = _find_card(_state, entity_id)
        if card is None:
            raise ValueError(f"Card '{entity_id}' not found.")
        card.setdefault("resources", []).append(new_res)
    else:
        raise ValueError(f"Unknown entity_type '{entity_type}'. Use epic, story, or card.")
    _save(_state)
    return {"id": new_res["id"]}


@mcp.tool()
def list_resources(entity_type: str = "all", entity_id: str = "") -> list[dict]:
    """List resources attached to entities.

    Args:
        entity_type: "epic", "story", "card", or "all"
        entity_id: specific entity id, or "" for all entities of that type
    """
    results = []

    def collect(etype, entity):
        for r in entity.get("resources", []):
            results.append({**r, "entity_type": etype, "entity_id": entity["id"], "entity_title": entity.get("title", "")})

    if entity_type in ("epic", "all"):
        for ep in _state["epics"]:
            if not entity_id or ep["id"] == entity_id:
                collect("epic", ep)
    if entity_type in ("story", "all"):
        for s in _state["stories"]:
            if not entity_id or s["id"] == entity_id:
                collect("story", s)
    if entity_type in ("card", "all"):
        for col in _state["columns"]:
            for card in col["cards"]:
                if not entity_id or card["id"] == entity_id:
                    collect("card", card)
    return results


@mcp.tool()
def delete_resource(resource_id: str) -> dict:
    """Delete a resource by id from whichever entity it belongs to.

    Args:
        resource_id: id of the resource to delete
    """
    for ep in _state["epics"]:
        for r in ep.get("resources", []):
            if r["id"] == resource_id:
                ep["resources"].remove(r)
                _save(_state)
                return {"deleted_resource_id": resource_id}
    for s in _state["stories"]:
        for r in s.get("resources", []):
            if r["id"] == resource_id:
                s["resources"].remove(r)
                _save(_state)
                return {"deleted_resource_id": resource_id}
    for col in _state["columns"]:
        for card in col["cards"]:
            for r in card.get("resources", []):
                if r["id"] == resource_id:
                    card["resources"].remove(r)
                    _save(_state)
                    return {"deleted_resource_id": resource_id}
    raise ValueError(f"Resource '{resource_id}' not found.")


# ── Board summary ─────────────────────────────────────────────────────────────

@mcp.tool()
def get_board_summary() -> str:
    """Return the full board as a Markdown table, grouped by column."""
    lines = ["# Kanban Board\n"]
    for col in _state["columns"]:
        lines.append(f"## {col['title']} ({len(col['cards'])} cards)\n")
        if col["cards"]:
            lines.append("| Title | Tag | Epic | Description |")
            lines.append("|-------|-----|------|-------------|")
            for card in col["cards"]:
                eid = card.get("epic_id") or ""
                epic_title = next((e["title"] for e in _state["epics"] if e["id"] == eid), "—")
                desc = (card["desc"] or "—").replace("|", "\\|")
                lines.append(f"| {card['title']} | {card['tag']} | {epic_title} | {desc} |")
        else:
            lines.append("_No cards_")
        lines.append("")
    return "\n".join(lines)


# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    mcp.run()
