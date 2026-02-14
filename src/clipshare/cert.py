import datetime
import ipaddress
import logging
import socket
from pathlib import Path

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.x509.oid import NameOID

log = logging.getLogger(__name__)


def _get_lan_ips() -> list[str]:
    ips: list[str] = []
    try:
        # Connect to a public address to find local IP (no data sent)
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
            s.connect(("8.8.8.8", 80))
            ips.append(s.getsockname()[0])
    except OSError:
        pass
    return ips


def ensure_cert(cert_dir: Path) -> tuple[Path, Path]:
    cert_path = cert_dir / "cert.pem"
    key_path = cert_dir / "key.pem"

    if cert_path.exists() and key_path.exists():
        log.info("Using existing certificate: %s", cert_path)
        return cert_path, key_path

    cert_dir.mkdir(parents=True, exist_ok=True)
    log.info("Generating self-signed certificate in %s", cert_dir)

    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)

    hostname = socket.gethostname()
    subject = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, hostname)])

    san_entries: list[x509.GeneralName] = [x509.DNSName(hostname), x509.DNSName("localhost")]
    for ip in _get_lan_ips():
        san_entries.append(x509.IPAddress(ipaddress.ip_address(ip)))

    now = datetime.datetime.now(datetime.UTC)
    cert = (
        x509.CertificateBuilder()
        .subject_name(subject)
        .issuer_name(subject)
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(now)
        .not_valid_after(now + datetime.timedelta(days=365))
        .add_extension(x509.SubjectAlternativeName(san_entries), critical=False)
        .sign(key, hashes.SHA256())
    )

    key_path.write_bytes(
        key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.TraditionalOpenSSL,
            encryption_algorithm=serialization.NoEncryption(),
        )
    )
    cert_path.write_bytes(cert.public_bytes(serialization.Encoding.PEM))

    log.info("Certificate generated: %s", cert_path)
    return cert_path, key_path
