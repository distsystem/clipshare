import tomllib
from pathlib import Path

import platformdirs
import pydantic

_APP_NAME = "clipshare"
_CONFIG_DIR = Path(platformdirs.user_config_dir(_APP_NAME))
_DATA_DIR = Path(platformdirs.user_data_dir(_APP_NAME))


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
    host: str = "0.0.0.0"
    port: int = 4243
    max_entries: int = 100
    db_path: Path = _DATA_DIR / "entries.db"
    cert_dir: Path = _DATA_DIR


def _load_toml() -> dict:
    path = _CONFIG_DIR / "config.toml"
    if not path.exists():
        return {}
    with open(path, "rb") as f:
        return tomllib.load(f)


def load_server_config() -> ServerConfig:
    raw = _load_toml().get("server", {})
    return ServerConfig.model_validate(raw)
