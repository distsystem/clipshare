"""CLI entry point: python -m clipshare server."""

import argparse
import logging


def main() -> None:
    parser = argparse.ArgumentParser(prog="clipshare", description="Distributed clipboard sync")
    parser.add_argument("command", choices=["server"], help="Run mode")
    parser.add_argument("-v", "--verbose", action="store_true", help="Debug logging")
    args = parser.parse_args()

    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
        datefmt="%H:%M:%S",
    )

    if args.command == "server":
        _run_server()


def _run_server() -> None:
    import uvicorn

    from clipshare.cert import ensure_cert
    from clipshare.config import load_server_config
    from clipshare.server import create_app

    config = load_server_config()
    cert_path, key_path = ensure_cert(config.cert_dir)

    app = create_app(config)

    uvicorn.run(
        app,
        host=config.host,
        port=config.port,
        ssl_certfile=str(cert_path),
        ssl_keyfile=str(key_path),
        log_level="info",
    )


if __name__ == "__main__":
    main()
