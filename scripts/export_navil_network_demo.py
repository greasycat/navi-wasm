#!/usr/bin/env python3
"""Export a compact Navil book snapshot for the WASM network demo."""

from __future__ import annotations

import argparse
import json
import os
from collections import defaultdict
from datetime import date, datetime
from pathlib import Path
from typing import Any

import psycopg
from psycopg.rows import dict_row


DEFAULT_BOOK_ID = "37699d66-c8c0-4e3c-ab05-bc2303de9699"
DEFAULT_VECTOR_SPACES_ORDER_INDEX = 3
DEFAULT_OUTPUT = Path(__file__).resolve().parents[1] / "demo/navil-network/data/ladw-network.json"


def normalize_database_url(url: str) -> str:
    if url.startswith("postgresql+psycopg://"):
        return "postgresql://" + url.removeprefix("postgresql+psycopg://")
    return url


def json_default(value: Any) -> str:
    if isinstance(value, (datetime, date)):
        return value.isoformat()
    return str(value)


def parse_json_field(raw: str | None, fallback: Any) -> Any:
    if not raw:
        return fallback
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return fallback


def parent_from_path(path: str | None) -> int | None:
    if not path or "." not in path:
        return None
    parent_label = path.split(".")[-2]
    if not parent_label.startswith("n"):
        return None
    try:
        return int(parent_label[1:])
    except ValueError:
        return None


def attach_parent_indices(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    stack: list[dict[str, Any]] = []
    by_order = {int(entry["order_index"]): entry for entry in entries}

    for entry in entries:
        order_index = int(entry["order_index"])
        parent_order_index = parent_from_path(entry.get("path"))
        if parent_order_index is None:
            while stack and int(stack[-1]["level"]) >= int(entry["level"]):
                stack.pop()
            parent_order_index = int(stack[-1]["order_index"]) if stack else None
        entry["parent_order_index"] = parent_order_index if parent_order_index in by_order else None
        stack.append(entry)

    child_counts: dict[int, int] = defaultdict(int)
    for entry in entries:
        parent_order_index = entry["parent_order_index"]
        if parent_order_index is not None:
            child_counts[int(parent_order_index)] += 1
    for entry in entries:
        entry["child_count"] = child_counts[int(entry["order_index"])]

    return entries


def as_float(value: Any) -> float | None:
    try:
        parsed = float(value)
    except (TypeError, ValueError):
        return None
    return parsed if parsed == parsed else None


def attach_relevance(entries: list[dict[str, Any]], ranked_toc: list[dict[str, Any]]) -> None:
    ranked_by_order = {
        int(row["order_index"]): row
        for row in ranked_toc
        if row.get("order_index") is not None
    }

    effective_values: list[float] = []
    by_order = {int(entry["order_index"]): entry for entry in entries}
    for entry in entries:
        order_index = int(entry["order_index"])
        ranked = ranked_by_order.get(order_index)
        direct = as_float(ranked.get("importance") if ranked else None)
        if direct is not None:
            effective = direct
            source = "ranked_toc"
            source_parent = None
        else:
            parent_order_index = entry.get("parent_order_index")
            parent = by_order.get(int(parent_order_index)) if parent_order_index is not None else None
            inherited = as_float(parent.get("effective_relevance") if parent else None)
            effective = inherited
            source = "parent" if inherited is not None else None
            source_parent = parent_order_index if inherited is not None else None

        entry["relevance"] = direct
        entry["effective_relevance"] = effective
        entry["normalized_relevance"] = None
        entry["relevance_source"] = source
        entry["relevance_parent_order_index"] = source_parent
        entry["relevance_rationale"] = ranked.get("rationale") if ranked else None
        entry["relevance_skip_rationale"] = ranked.get("skip_rationale") if ranked else None
        if effective is not None:
            effective_values.append(effective)

    if not effective_values:
        return

    lo = min(effective_values)
    hi = max(effective_values)
    span = hi - lo
    for entry in entries:
        effective = as_float(entry.get("effective_relevance"))
        if effective is None:
            entry["normalized_relevance"] = 0.0
        elif span <= 0:
            entry["normalized_relevance"] = 1.0
        else:
            entry["normalized_relevance"] = (effective - lo) / span


def entry_label(entry: dict[str, Any]) -> str:
    return f"{entry['order_index']}: {entry['title']}"


def descendant_entries(entries: list[dict[str, Any]], root_order_index: int) -> list[dict[str, Any]]:
    root = next((entry for entry in entries if int(entry["order_index"]) == root_order_index), None)
    if root is None:
        raise SystemExit(f"TOC order index not found: {root_order_index}")
    root_path = root.get("path") or f"n{root_order_index}"
    prefix = f"{root_path}."
    return [
        entry
        for entry in entries
        if entry.get("path") == root_path or str(entry.get("path", "")).startswith(prefix)
    ]


def next_sibling_page(entries: list[dict[str, Any]], root: dict[str, Any]) -> int | None:
    root_order_index = int(root["order_index"])
    root_level = int(root["level"])
    for entry in entries:
        if int(entry["order_index"]) <= root_order_index:
            continue
        if int(entry["level"]) <= root_level:
            return int(entry["page_number"])
    return None


def attach_component_parent(
    component: dict[str, Any],
    subgraph_entries: list[dict[str, Any]],
    fallback_order_index: int,
) -> int:
    page_no = int(component["page_no"])
    candidates = [
        entry
        for entry in subgraph_entries
        if int(entry["page_number"]) <= page_no
    ]
    if not candidates:
        return fallback_order_index
    candidates.sort(key=lambda entry: (int(entry["page_number"]), int(entry["level"]), int(entry["order_index"])))
    return int(candidates[-1]["order_index"])


def export_refined_component_subgraph(
    cur: psycopg.Cursor[dict[str, Any]],
    book_id: str,
    entries: list[dict[str, Any]],
    root_order_index: int,
) -> dict[str, Any]:
    root = next(entry for entry in entries if int(entry["order_index"]) == root_order_index)
    subgraph_entries = descendant_entries(entries, root_order_index)
    page_start = int(root["page_number"])
    next_page = next_sibling_page(entries, root)
    page_end = (next_page - 1) if next_page is not None else max(int(entry["page_number"]) for entry in subgraph_entries)

    cur.execute(
        """
        SELECT
            id,
            stage_index,
            toc_entry_order_index,
            page_no,
            sequence,
            role,
            text,
            description,
            note,
            source_component_ids,
            bbox,
            extra
        FROM book_component_refinements
        WHERE book_id = %s AND page_no BETWEEN %s AND %s
        ORDER BY page_no, sequence, id
        """,
        (book_id, page_start, page_end),
    )
    components = [dict(row) for row in cur.fetchall()]
    for component in components:
        component["parent_toc_order_index"] = attach_component_parent(component, subgraph_entries, root_order_index)
        text = (component.get("text") or "").strip()
        description = (component.get("description") or "").strip()
        component["preview"] = description or (text[:180] + ("..." if len(text) > 180 else ""))
        if len(text) > 500:
            component["text"] = text[:500] + "..."

    return {
        "id": "chapter-1-vector-spaces-refined",
        "title": "Chapter 1: Vector spaces refined components",
        "root_order_index": root_order_index,
        "root_title": root["title"],
        "page_start": page_start,
        "page_end": page_end,
        "toc_order_indices": [int(entry["order_index"]) for entry in subgraph_entries],
        "components": components,
    }


def query_fixture(database_url: str, book_id: str) -> dict[str, Any]:
    with psycopg.connect(database_url, row_factory=dict_row) as conn:
        with conn.cursor() as cur:
            cur.execute(
                """
                SELECT
                    b.id,
                    b.title,
                    b.filename,
                    b.toc_page_offset,
                    b.created_at,
                    b.updated_at
                FROM books b
                WHERE b.id = %s
                """,
                (book_id,),
            )
            book = cur.fetchone()
            if book is None:
                raise SystemExit(f"Book not found: {book_id}")

            cur.execute(
                """
                SELECT
                    order_index,
                    level,
                    title,
                    page_number,
                    path::text AS path,
                    is_problem
                FROM book_toc_entries
                WHERE book_id = %s
                ORDER BY order_index
                """,
                (book_id,),
            )
            entries = [dict(row) for row in cur.fetchall()]

            cur.execute(
                """
                SELECT
                    toc_entry_order_index AS order_index,
                    COUNT(*)::int AS chunk_count,
                    MIN(page_start) AS chunk_page_start,
                    MAX(page_end) AS chunk_page_end
                FROM book_chunks
                WHERE book_id = %s AND toc_entry_order_index IS NOT NULL
                GROUP BY toc_entry_order_index
                """,
                (book_id,),
            )
            chunks_by_order = {int(row["order_index"]): dict(row) for row in cur.fetchall()}

            cur.execute(
                """
                SELECT
                    toc_entry_order_index AS order_index,
                    role,
                    COUNT(*)::int AS count
                FROM book_component_refinements
                WHERE book_id = %s
                GROUP BY toc_entry_order_index, role
                ORDER BY toc_entry_order_index, role
                """,
                (book_id,),
            )
            role_counts: dict[int, dict[str, int]] = defaultdict(dict)
            for row in cur.fetchall():
                role_counts[int(row["order_index"])][str(row["role"])] = int(row["count"])

            cur.execute(
                """
                SELECT ranked_toc, path_stages, created_at
                FROM book_paths
                WHERE book_id = %s
                """,
                (book_id,),
            )
            path_row = cur.fetchone()

            entries = attach_parent_indices(entries)
            ranked_toc = parse_json_field(path_row["ranked_toc"] if path_row else None, [])
            path_stages = parse_json_field(path_row["path_stages"] if path_row else None, [])
            attach_relevance(entries, ranked_toc)
            component_subgraphs = [
                export_refined_component_subgraph(cur, book_id, entries, DEFAULT_VECTOR_SPACES_ORDER_INDEX)
            ]

    page_numbers = [int(entry["page_number"]) for entry in entries if entry.get("page_number") is not None]

    for entry in entries:
        order_index = int(entry["order_index"])
        chunk_row = chunks_by_order.get(order_index, {})
        roles = role_counts.get(order_index, {})
        entry["chunk_count"] = int(chunk_row.get("chunk_count") or 0)
        entry["chunk_page_start"] = chunk_row.get("chunk_page_start")
        entry["chunk_page_end"] = chunk_row.get("chunk_page_end")
        entry["refinement_count"] = sum(roles.values())
        entry["roles"] = roles

    return {
        "schema_version": 1,
        "source": {
            "app": "navil",
            "book_id": book_id,
            "exported_from": "scripts/export_navil_network_demo.py",
        },
        "book": {
            **dict(book),
            "toc_entry_count": len(entries),
            "page_start": min(page_numbers) if page_numbers else None,
            "page_end": max(page_numbers) if page_numbers else None,
        },
        "entries": entries,
        "path": {
            "created_at": path_row["created_at"] if path_row else None,
            "ranked_toc": ranked_toc,
            "path_stages": path_stages,
        },
        "component_subgraphs": component_subgraphs,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--book-id", default=DEFAULT_BOOK_ID)
    parser.add_argument(
        "--database-url",
        default=os.environ.get("DATABASE_URL"),
        help="Postgres URL. Defaults to DATABASE_URL. SQLAlchemy postgresql+psycopg URLs are accepted.",
    )
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    if not args.database_url:
        raise SystemExit("Set DATABASE_URL or pass --database-url.")

    data = query_fixture(normalize_database_url(args.database_url), args.book_id)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(data, indent=2, default=json_default) + "\n", encoding="utf-8")
    print(f"Wrote {args.output} ({len(data['entries'])} TOC entries)")


if __name__ == "__main__":
    main()
