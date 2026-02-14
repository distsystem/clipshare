"""Cross-compile clipshare-daemon for multiple targets via cargo-zigbuild."""

import shutil
import subprocess
import sys
from pathlib import Path

TARGETS = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-gnu",
]

PROJECT_ROOT = Path(__file__).resolve().parent.parent
MANIFEST = PROJECT_ROOT / "daemon" / "Cargo.toml"
DIST = PROJECT_ROOT / "dist"


def _binary_name(target: str) -> str:
    return "clipshare-daemon.exe" if "windows" in target else "clipshare-daemon"


def ensure_targets() -> None:
    result = subprocess.run(
        ["rustup", "target", "list", "--installed"],
        capture_output=True,
        text=True,
        check=True,
    )
    installed = set(result.stdout.splitlines())
    for target in TARGETS:
        if target not in installed:
            print(f"  Adding target: {target}")
            subprocess.run(["rustup", "target", "add", target], check=True)


def build(target: str) -> Path | None:
    print(f"\n>> Building {target}")
    ret = subprocess.run(
        [
            "cargo",
            "zigbuild",
            "--release",
            "--target",
            target,
            "--manifest-path",
            str(MANIFEST),
        ],
    )
    if ret.returncode != 0:
        print(f"  FAILED: {target}")
        return None

    src = (
        PROJECT_ROOT
        / "daemon"
        / "target"
        / target
        / "release"
        / _binary_name(target)
    )
    dst_dir = DIST / target
    dst_dir.mkdir(parents=True, exist_ok=True)
    dst = dst_dir / _binary_name(target)
    shutil.copy2(src, dst)
    print(f"  -> {dst.relative_to(PROJECT_ROOT)}")
    return dst


def main() -> None:
    print("Ensuring rustup targets...")
    ensure_targets()

    results: dict[str, bool] = {}
    for target in TARGETS:
        out = build(target)
        results[target] = out is not None

    print("\n== Build Summary ==")
    ok = sum(results.values())
    for target, success in results.items():
        status = "OK" if success else "FAIL"
        print(f"  [{status}] {target}")
    print(f"\n{ok}/{len(TARGETS)} targets built successfully.")

    if ok < len(TARGETS):
        sys.exit(1)


if __name__ == "__main__":
    main()
