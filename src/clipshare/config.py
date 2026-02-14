import tomllib
from pathlib import Path

import pydantic


class MimeContent(pydantic.BaseModel):
    mime_type: str
    data: str


class ClipboardEntry(pydantic.BaseModel):
    id: str
    source_host: str
    timestamp_ms: int
    contents: list[MimeContent]
    text_preview: str


class ServerConfig(pydantic.BaseModel):
    model_config = pydantic.ConfigDict(validate_default=True)

    host: str = "0.0.0.0"
    port: int = 8443
    max_entries: int = 100
    db_path: Path = Path("~/.local/share/clipshare/entries.db")
    cert_dir: Path = Path("~/.local/share/clipshare")

    @pydantic.field_validator("db_path", "cert_dir", mode="after")
    @classmethod
    def _expand_user(cls, v: Path) -> Path:
        return v.expanduser()


_CONFIG_PATH = Path("~/.config/clipshare/config.toml")


def _load_toml() -> dict:
    path = _CONFIG_PATH.expanduser()
    if not path.exists():
        return {}
    with open(path, "rb") as f:
        return tomllib.load(f)


def load_server_config() -> ServerConfig:
    raw = _load_toml().get("server", {})
    return ServerConfig.model_validate(raw)
