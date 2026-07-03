#!/usr/bin/env python3
"""
HomeKit QR Code URI generator (matches esp-homekit-sdk esp_hap_get_setup_payload).

Usage:
  ./generate_qr.py                          # defaults from sdkconfig.defaults
  ./generate_qr.py -s                       # URI only
  ./generate_qr.py 111-22-334 8 ES32        # explicit setup code, category, setup id
  ./generate_qr.py --wac 111-22-334 8 ES32  # include WAC flag (MFi only)
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

BASE36 = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"

# esp_hap_setup_payload.c
HAP_OVER_IP_MASK = 0x10000000
WAC_MASK = 0x40000000
SETUP_PAYLOAD_PREFIX = "X-HM://00"


def base36_encode(number: int) -> str:
    if number == 0:
        return "0"
    chars: list[str] = []
    n = number
    while n:
        n, i = divmod(n, 36)
        chars.append(BASE36[i])
    return "".join(reversed(chars))


def generate_payload(category: int, setup_code: int, *, hap_over_ip: bool = True, wac: bool = False) -> str:
    """Bit layout matches esp-homekit-sdk esp_hap_setup_payload.c."""
    payload = setup_code & 0x7FFFFFF
    payload |= (category & 0xFF) << 31
    if hap_over_ip:
        payload |= HAP_OVER_IP_MASK
    if wac:
        payload |= WAC_MASK
    return base36_encode(payload).upper()


def build_uri(setup_code: str, category: int, setup_id: str, *, wac: bool = False) -> str:
    code_digits = setup_code.replace("-", "")
    if len(code_digits) != 8 or not code_digits.isdigit():
        raise ValueError(f"Setup code must be 8 digits (e.g. 111-22-334), got {setup_code!r}")
    if len(setup_id) != 4:
        raise ValueError(f"Setup ID must be exactly 4 characters, got {setup_id!r}")

    payload = generate_payload(category, int(code_digits), wac=wac)
    return f"{SETUP_PAYLOAD_PREFIX}{payload}{setup_id.upper()}"


def parse_sdkconfig_defaults(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        return values
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        m = re.match(r"^CONFIG_(\w+)=(.+)$", line)
        if not m:
            continue
        key, raw = m.group(1), m.group(2).strip()
        if raw.startswith('"') and raw.endswith('"'):
            raw = raw[1:-1]
        values[key] = raw
    return values


def defaults_from_sdkconfig(repo_root: Path) -> tuple[str, int, str]:
    cfg = parse_sdkconfig_defaults(repo_root / "sdkconfig.defaults")
    setup_code = cfg.get("EXAMPLE_SETUP_CODE", "111-22-334")
    setup_id = cfg.get("EXAMPLE_SETUP_ID", "ES32")
    # HAP_CID_SWITCH == 8 in esp-homekit-sdk hap.h
    category = 8
    return setup_code, category, setup_id


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    default_code, default_category, default_setup_id = defaults_from_sdkconfig(repo_root)

    parser = argparse.ArgumentParser(
        description="Generate HomeKit pairing QR URI (esp-homekit-sdk compatible)",
    )
    parser.add_argument(
        "setup_code",
        nargs="?",
        default=default_code,
        help=f"Setup code with dashes (default from sdkconfig: {default_code})",
    )
    parser.add_argument(
        "category",
        type=int,
        nargs="?",
        default=default_category,
        help=f"Accessory category id (default: {default_category} = Switch)",
    )
    parser.add_argument(
        "setup_id",
        nargs="?",
        default=default_setup_id,
        help=f"4-char Setup ID — must match firmware (default: {default_setup_id})",
    )
    parser.add_argument(
        "--wac",
        action="store_true",
        help="Set WAC flag (MFi/WAC accessories only)",
    )
    parser.add_argument(
        "-s",
        "--silent",
        action="store_true",
        help="Print URI only",
    )

    args = parser.parse_args()

    try:
        uri = build_uri(args.setup_code, args.category, args.setup_id, wac=args.wac)
    except ValueError as exc:
        parser.error(str(exc))

    if args.silent:
        print(uri)
        return 0

    code_digits = args.setup_code.replace("-", "")
    payload = generate_payload(args.category, int(code_digits), wac=args.wac)

    print("==========================================")
    print(" HomeKit QR Code URI Generator")
    print(" (esp-homekit-sdk esp_hap_get_setup_payload)")
    print("==========================================")
    print(" Settings:")
    print(f"   Setup Code : {args.setup_code} ({code_digits})")
    print(f"   Category   : {args.category} (8 = Switch / HAP_CID_SWITCH)")
    print(f"   Setup ID   : {args.setup_id}  ← must match CONFIG_EXAMPLE_SETUP_ID")
    print(f"   HAP over IP: yes (bit 28)")
    print(f"   WAC        : {'yes' if args.wac else 'no'}")
    print("------------------------------------------")
    print(f" Payload     : {payload}")
    print(f" QR URI      : {uri}")
    if args.setup_id.upper() != default_setup_id.upper():
        expected = build_uri(args.setup_code, args.category, default_setup_id, wac=args.wac)
        print("------------------------------------------")
        print(f" WARNING: sdkconfig.defaults uses Setup ID {default_setup_id!r}")
        print(f" Firmware URI: {expected}")
    print("==========================================")
    return 0


if __name__ == "__main__":
    sys.exit(main())
