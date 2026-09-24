#!/usr/bin/env python3
"""Normalize LLVM LCOV paths and emit independent API and worker reports."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "target" / "sonar-rust-lcov.info"
if not SOURCE.is_file():
    raise SystemExit(f"Rust LCOV report is missing: {SOURCE}")

records: list[list[str]] = []
record: list[str] = []
for line in SOURCE.read_text(encoding="utf-8").splitlines():
    if line.startswith("SF:"):
        source = Path(line[3:])
        if source.is_absolute():
            try:
                source = source.resolve().relative_to(ROOT)
            except ValueError as error:
                raise SystemExit(f"LCOV source is outside repository: {source}") from error
        line = f"SF:{source.as_posix()}"
    record.append(line)
    if line == "end_of_record":
        records.append(record)
        record = []
if record:
    records.append(record)

SOURCE.write_text("\n".join(line for item in records for line in item) + "\n", encoding="utf-8")
for service, prefix in (("api", "notification-api/src/"), ("worker", "notification-worker/src/")):
    scoped = [item for item in records if any(line.startswith("SF:" + prefix) for line in item)]
    output = ROOT / "target" / f"sonar-{service}-rust-lcov.info"
    output.write_text("\n".join(line for item in scoped for line in item) + "\n", encoding="utf-8")
    if not scoped:
        raise SystemExit(f"No LCOV records found for {service}")
