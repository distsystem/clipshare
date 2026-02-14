import hashlib
import json
from pathlib import Path

import aiosqlite

from clipshare.config import ClipboardEntry, MimeContent

_CREATE_TABLE = """
CREATE TABLE IF NOT EXISTS entries (
    id TEXT PRIMARY KEY,
    source_host TEXT NOT NULL,
    timestamp_ms INTEGER NOT NULL,
    contents_json TEXT NOT NULL,
    text_preview TEXT NOT NULL,
    content_hash TEXT NOT NULL
)
"""


class EntryStore:
    def __init__(self, db_path: Path, max_entries: int = 100) -> None:
        self._db_path = db_path
        self._max_entries = max_entries
        self._db: aiosqlite.Connection | None = None

    async def init(self) -> None:
        self._db_path.parent.mkdir(parents=True, exist_ok=True)
        self._db = await aiosqlite.connect(str(self._db_path))
        await self._db.execute("PRAGMA journal_mode=WAL")
        await self._db.execute(_CREATE_TABLE)
        await self._db.execute(
            "CREATE INDEX IF NOT EXISTS idx_content_hash ON entries(content_hash)"
        )
        await self._db.commit()

    async def close(self) -> None:
        if self._db:
            await self._db.close()

    async def add(self, entry: ClipboardEntry) -> bool:
        """Add entry. Returns False if content was deduplicated (timestamp refreshed)."""
        contents_json = json.dumps([c.model_dump() for c in entry.contents])
        content_hash = _hash_contents(contents_json)

        cursor = await self._db.execute(
            "SELECT id FROM entries WHERE content_hash = ? LIMIT 1",
            (content_hash,),
        )
        existing = await cursor.fetchone()
        if existing:
            await self._db.execute(
                "UPDATE entries SET timestamp_ms = ?, source_host = ? WHERE id = ?",
                (entry.timestamp_ms, entry.source_host, existing[0]),
            )
            await self._db.commit()
            return False

        await self._db.execute(
            "INSERT INTO entries (id, source_host, timestamp_ms, contents_json, text_preview, content_hash) "
            "VALUES (?, ?, ?, ?, ?, ?)",
            (entry.id, entry.source_host, entry.timestamp_ms, contents_json, entry.text_preview, content_hash),
        )
        await self._db.commit()
        await self._evict()
        return True

    async def list(self, limit: int = 50, offset: int = 0) -> list[ClipboardEntry]:
        cursor = await self._db.execute(
            "SELECT id, source_host, timestamp_ms, contents_json, text_preview "
            "FROM entries ORDER BY timestamp_ms DESC LIMIT ? OFFSET ?",
            (limit, offset),
        )
        rows = await cursor.fetchall()
        return [_row_to_entry(r) for r in rows]

    async def get(self, entry_id: str) -> ClipboardEntry | None:
        cursor = await self._db.execute(
            "SELECT id, source_host, timestamp_ms, contents_json, text_preview "
            "FROM entries WHERE id = ?",
            (entry_id,),
        )
        row = await cursor.fetchone()
        return _row_to_entry(row) if row else None

    async def delete(self, entry_id: str) -> bool:
        cursor = await self._db.execute("DELETE FROM entries WHERE id = ?", (entry_id,))
        await self._db.commit()
        return cursor.rowcount > 0

    async def _evict(self) -> None:
        await self._db.execute(
            "DELETE FROM entries WHERE id NOT IN "
            "(SELECT id FROM entries ORDER BY timestamp_ms DESC LIMIT ?)",
            (self._max_entries,),
        )
        await self._db.commit()


def _row_to_entry(row: tuple) -> ClipboardEntry:
    return ClipboardEntry(
        id=row[0],
        source_host=row[1],
        timestamp_ms=row[2],
        contents=[MimeContent.model_validate(c) for c in json.loads(row[3])],
        text_preview=row[4],
    )


def _hash_contents(contents_json: str) -> str:
    return hashlib.sha256(contents_json.encode()).hexdigest()
