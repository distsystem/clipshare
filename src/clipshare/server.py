"""FastAPI server with REST API, WebSocket, and static files."""

import json
import logging
import time
import uuid
from contextlib import asynccontextmanager
from importlib.resources import files

import fastapi
import pydantic
from fastapi import WebSocket, WebSocketDisconnect
from fastapi.responses import RedirectResponse
from fastapi.staticfiles import StaticFiles

from clipshare.config import (
    ClipboardEntry,
    MimeContent,
    ServerConfig,
    load_server_config,
)
from clipshare.storage import EntryStore

log = logging.getLogger(__name__)

_store: EntryStore
_ws_connections: dict[str, tuple[WebSocket, str]] = {}  # conn_id → (ws, host)


class CreateEntryRequest(pydantic.BaseModel):
    source_host: str = "unknown"
    contents: list[MimeContent]


@asynccontextmanager
async def _lifespan(app: fastapi.FastAPI):
    global _store
    cfg = app.state.config
    _store = EntryStore(cfg.db_path, cfg.max_entries)
    await _store.init()
    log.info("Storage initialized: %s", cfg.db_path)
    yield
    await _store.close()


def create_app(config: ServerConfig | None = None) -> fastapi.FastAPI:
    if config is None:
        config = load_server_config()

    app = fastapi.FastAPI(title="ClipShare", lifespan=_lifespan)
    app.state.config = config

    app.include_router(_router)

    static_dir = str(files("clipshare").joinpath("static"))
    app.mount("/static", StaticFiles(directory=static_dir), name="static")

    return app


_router = fastapi.APIRouter()


@_router.get("/")
async def index():
    return RedirectResponse("/static/index.html")


@_router.post("/api/entries")
async def create_entry(body: CreateEntryRequest) -> dict | ClipboardEntry:
    text_preview = ""
    for c in body.contents:
        if c.mime_type == "text/plain":
            text_preview = c.data[:200]
            break

    entry = ClipboardEntry(
        id=str(uuid.uuid4()),
        source_host=body.source_host,
        timestamp_ms=int(time.time() * 1000),
        contents=body.contents,
        text_preview=text_preview,
    )
    is_new = await _store.add(entry)
    if not is_new:
        log.debug("Duplicate content from %s, skipped", entry.source_host)
        return {"ok": True, "duplicate": True}
    log.info("New entry %s from %s", entry.id, entry.source_host)
    await _broadcast(entry)
    return entry


@_router.get("/api/entries")
async def list_entries(limit: int = 50, offset: int = 0) -> list[ClipboardEntry]:
    return await _store.list(limit=limit, offset=offset)


@_router.get("/api/entries/{entry_id}")
async def get_entry(entry_id: str) -> ClipboardEntry:
    entry = await _store.get(entry_id)
    if entry is None:
        raise fastapi.HTTPException(status_code=404, detail="Entry not found")
    return entry


@_router.delete("/api/entries/{entry_id}")
async def delete_entry(entry_id: str):
    deleted = await _store.delete(entry_id)
    if not deleted:
        raise fastapi.HTTPException(status_code=404, detail="Entry not found")
    return {"ok": True}


@_router.websocket("/ws")
async def websocket_endpoint(ws: WebSocket, host: str = "unknown"):
    await ws.accept()
    conn_id = str(uuid.uuid4())
    _ws_connections[conn_id] = (ws, host)
    log.info("WebSocket connected: %s (host=%s)", conn_id, host)
    try:
        while True:
            await ws.receive_text()  # keep alive
    except WebSocketDisconnect:
        pass
    finally:
        _ws_connections.pop(conn_id, None)
        log.info("WebSocket disconnected: %s", conn_id)


async def _broadcast(entry: ClipboardEntry) -> None:
    msg = json.dumps({"type": "new_entry", "entry": entry.model_dump()})
    dead: list[str] = []
    for conn_id, (ws, host) in _ws_connections.items():
        if host == entry.source_host:
            continue
        try:
            await ws.send_text(msg)
        except Exception:
            dead.append(conn_id)
    for conn_id in dead:
        _ws_connections.pop(conn_id, None)
