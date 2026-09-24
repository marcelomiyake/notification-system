#!/usr/bin/env python3
"""Enforce the repository's SonarQube thresholds when server gate admin is unavailable."""

from __future__ import annotations

import base64
import json
import os
import sys
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode
from urllib.request import Request, urlopen
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
GATE_FILE = ROOT / ".sonar" / "quality-gate.json"
METRICS = (
    "coverage",
    "duplicated_lines_density",
    "new_blocker_violations",
    "new_critical_violations",
    "security_hotspots",
    "security_hotspots_reviewed",
)


def fetch_measures(host: str, token: str, project: str) -> dict[str, float]:
    query = urlencode({"component": project, "metricKeys": ",".join(METRICS)})
    request = Request(f"{host.rstrip('/')}/api/measures/component?{query}")
    credentials = base64.b64encode(f"{token}:".encode()).decode()
    request.add_header("Authorization", f"Basic {credentials}")
    try:
        with urlopen(request, timeout=15) as response:
            payload = json.load(response)
    except (HTTPError, URLError, TimeoutError) as error:
        raise RuntimeError(f"could not read SonarQube measures for {project}: {error}") from error

    measures: dict[str, float] = {}
    for measure in payload.get("component", {}).get("measures", []):
        raw = measure.get("period", {}).get("value", measure.get("value"))
        if raw is not None:
            measures[measure["metric"]] = float(raw)
    return measures


def main() -> int:
    host = os.environ.get("SONAR_HOST_URL", "http://127.0.0.1:9000")
    token = os.environ.get("SONAR_TOKEN")
    if not token:
        print("Set SONAR_TOKEN in the environment; it is never read from a file.", file=sys.stderr)
        return 2

    gate = json.loads(GATE_FILE.read_text(encoding="utf-8"))
    failed = False
    for project in gate["projects"]:
        measures = fetch_measures(host, token, project)
        violations: list[str] = []
        for condition in gate["conditions"]:
            metric = condition["metric"]
            value = measures.get(metric, 0.0)
            threshold = float(condition["threshold"])
            operator = condition["operator"]
            fails = (operator == "LT" and value < threshold) or (
                operator == "GTE" and value >= threshold
            ) or (operator == "GT" and value > threshold)
            if fails:
                violations.append(f"{metric}={value:g} violates {operator} {threshold:g}")

        hotspots = measures.get("security_hotspots", 0.0)
        reviewed = measures.get("security_hotspots_reviewed", 0.0)
        if reviewed < hotspots:
            violations.append(f"security hotspots reviewed={reviewed:g}/{hotspots:g}")

        outcome = "FAIL" if violations else "PASS"
        print(
            f"{outcome} {project}: coverage={measures.get('coverage', 0):g}%, "
            f"duplication={measures.get('duplicated_lines_density', 0):g}%, "
            f"new blockers={measures.get('new_blocker_violations', 0):g}, "
            f"new critical={measures.get('new_critical_violations', 0):g}, "
            f"hotspots reviewed={reviewed:g}/{hotspots:g}"
        )
        for violation in violations:
            print(f"  - {violation}")
        failed |= bool(violations)

    return 1 if failed else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, RuntimeError) as error:
        print(error, file=sys.stderr)
        raise SystemExit(2)
