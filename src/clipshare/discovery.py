import asyncio
import json
import logging
import socket

log = logging.getLogger(__name__)

MULTICAST_GROUP = "239.255.42.1"
MULTICAST_PORT = 4243


async def run_broadcaster(port: int, protocol: str = "https") -> None:
    payload = json.dumps(
        {"service": "clipshare", "port": port, "protocol": protocol}
    ).encode()

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)

    try:
        while True:
            sock.sendto(payload, (MULTICAST_GROUP, MULTICAST_PORT))
            log.debug("Broadcast sent to %s:%d", MULTICAST_GROUP, MULTICAST_PORT)
            await asyncio.sleep(5)
    finally:
        sock.close()
