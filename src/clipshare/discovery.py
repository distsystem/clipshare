import asyncio
import fcntl
import json
import logging
import socket
import struct

log = logging.getLogger(__name__)

BROADCAST_PORT = 4243


def _broadcast_addrs() -> list[str]:
    addrs = []
    for _, name in socket.if_nameindex():
        if name == "lo":
            continue
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            result = fcntl.ioctl(
                s.fileno(),
                0x8919,  # SIOCGIFBRDADDR
                struct.pack("256s", name.encode()),
            )
            addr = socket.inet_ntoa(result[20:24])
            s.close()
            if addr not in ("0.0.0.0", "255.255.255.255"):
                addrs.append(addr)
        except OSError:
            continue
    return addrs or ["255.255.255.255"]


async def run_broadcaster(port: int, protocol: str = "https") -> None:
    payload = json.dumps(
        {"service": "clipshare", "port": port, "protocol": protocol}
    ).encode()

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)

    try:
        while True:
            for addr in _broadcast_addrs():
                sock.sendto(payload, (addr, BROADCAST_PORT))
                log.debug("Broadcast sent to %s:%d", addr, BROADCAST_PORT)
            await asyncio.sleep(5)
    finally:
        sock.close()
