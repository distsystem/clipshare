import logging
import socket
from contextlib import asynccontextmanager

import ipaddress

import ifaddr
import zeroconf
import zeroconf.asyncio

log = logging.getLogger(__name__)

SERVICE_TYPE = "_clipshare._tcp.local."

_RFC1918 = (
    ipaddress.ip_network("10.0.0.0/8"),
    ipaddress.ip_network("172.16.0.0/12"),
    ipaddress.ip_network("192.168.0.0/16"),
)


def _lan_addresses() -> list[bytes]:
    addrs = []
    for adapter in ifaddr.get_adapters():
        for ip in adapter.ips:
            if not isinstance(ip.ip, str):
                continue
            addr = ipaddress.ip_address(ip.ip)
            if any(addr in net for net in _RFC1918):
                addrs.append(addr.packed)
    return addrs or [socket.inet_aton("127.0.0.1")]


@asynccontextmanager
async def mdns_service(port: int, protocol: str = "https"):
    hostname = socket.gethostname()
    instance_name = f"{hostname}.{SERVICE_TYPE}"

    info = zeroconf.ServiceInfo(
        type_=SERVICE_TYPE,
        name=instance_name,
        port=port,
        properties={"protocol": protocol},
        server=f"{hostname}.local.",
        addresses=_lan_addresses(),
    )

    aiozc = zeroconf.asyncio.AsyncZeroconf()
    await aiozc.async_register_service(info)
    log.info("mDNS service registered: %s on port %d", instance_name, port)

    try:
        yield
    finally:
        await aiozc.async_unregister_service(info)
        await aiozc.async_close()
        log.info("mDNS service unregistered")
