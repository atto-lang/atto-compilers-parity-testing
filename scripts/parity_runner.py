#!/usr/bin/env python3
"""Fail-closed cross-backend Oreslang parity runner.

Audits and, when explicitly enabled, executes both:
  * legacy single-file candidates from manifest.json
  * multi-file checks from checks/manifest.json

Configure each backend independently with ORES_PARITY_<BACKEND>_COMPILE
and ORES_PARITY_<BACKEND>_RUN command templates. Template tokens:
{source} {artifact} {workdir}. Commands are tokenized with shlex and never
executed through a shell.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
BACKENDS = ("llvm", "java", "wasm", "dotnet", "js")
ENV_NAMES = {"llvm": "LLVM", "java": "JAVA", "wasm": "WASM",
             "dotnet": "DOTNET", "js": "JS"}
LOWERCASE_CHECK = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
IMPORT_RE = re.compile(r'^\s*import\s+.+?\s+from\s+"([^"]+)"\s*;', re.MULTILINE)


def safe_file(path_text):
    path = (ROOT / path_text).resolve()
    if ROOT not in path.parents or not path.is_file():
        raise SystemExit("Invalid source path: " + path_text)
    return path


def fingerprint(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def local_imports(path):
    result = []
    for target in IMPORT_RE.findall(path.read_text()):
        if not target.startswith("."):
            continue
        candidate = (path.parent / target).resolve()
        if candidate.suffix == "":
            candidate = candidate.with_suffix(".ores")
        if ROOT not in candidate.parents or not candidate.is_file():
            raise SystemExit(f"Invalid local import from {path.relative_to(ROOT)}: {target}")
        result.append(candidate)
    return result


def reachable_graph(entry):
    pending = [entry]
    seen = set()
    while pending:
        current = pending.pop()
        if current in seen:
            continue
        seen.add(current)
        pending.extend(local_imports(current))
    return sorted(seen, key=lambda p: p.relative_to(ROOT).as_posix())


def fingerprint_graph(paths):
    digest = hashlib.sha256()
    for path in paths:
        rel = path.relative_to(ROOT).as_posix().encode()
        digest.update(len(rel).to_bytes(4, "big"))
        digest.update(rel)
        data = path.read_bytes()
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
    return digest.hexdigest()


def command(template, source, artifact, workdir):
    return [part.format(source=str(source), artifact=str(artifact),
                        workdir=str(workdir)) for part in shlex.split(template)]


def invoke(argv, workdir, timeout):
    started = time.monotonic()
    try:
        cp = subprocess.run(argv, cwd=workdir, capture_output=True,
                            timeout=timeout, check=False)
        return {"status": "completed", "exit_code": cp.returncode,
                "stdout_hex": cp.stdout.hex(), "stderr_hex": cp.stderr.hex(),
                "duration_seconds": round(time.monotonic()-started, 4)}
    except subprocess.TimeoutExpired:
        return {"status": "timeout", "duration_seconds": round(time.monotonic()-started, 4)}
    except OSError as e:
        return {"status": "infrastructure_error", "error": str(e)}


def load_cases():
    legacy = json.loads((ROOT / "manifest.json").read_text())
    legacy_cases = []
    for c in legacy["cases"]:
        source = safe_file(c["path"])
        legacy_cases.append({
            "suite": "legacy",
            "case_id": c["id"],
            "name": c["name"],
            "source": source,
            "source_sha256": fingerprint(source),
            "expected_kind": c["expected_kind"],
            "expected_value": c.get("expected_value"),
            "validation_status": c["validation_status"],
            "feature_coverage": c["feature_coverage"],
        })

    checks_manifest = json.loads((ROOT / "checks" / "manifest.json").read_text())
    check_cases = []
    negative_count = 0
    manifest_names = []
    for item in checks_manifest["checks"]:
        expectation_path = safe_file(item["expectation"])
        expectation = json.loads(expectation_path.read_text())
        name = item["name"]
        manifest_names.append(name)
        if expectation.get("name") != name:
            raise SystemExit(f"Check name mismatch: {name}")
        if not LOWERCASE_CHECK.fullmatch(name):
            raise SystemExit(f"Check name must be lowercase kebab-case: {name}")
        entry = safe_file(expectation["entry"])
        check_dir = (ROOT / "checks" / name).resolve()
        if entry.parent != check_dir:
            raise SystemExit(f"Entry is outside its check directory: {expectation['entry']}")
        direct_import_files = sorted(set(local_imports(entry)))
        expected_imports = expectation.get("imported_files")
        if expected_imports is not None and len(direct_import_files) != expected_imports:
            raise SystemExit(
                f"{name}: main imports {len(direct_import_files)} local files, expected {expected_imports}")
        graph = reachable_graph(entry)
        if len(graph) < 4:
            raise SystemExit(f"{name}: reachable source graph must contain at least four files")
        check_cases.append({
            "suite": "checks",
            "case_id": name,
            "name": name,
            "source": entry,
            "source_sha256": fingerprint(entry),
            "graph_sha256": fingerprint_graph(graph),
            "graph_files": [p.relative_to(ROOT).as_posix() for p in graph],
            "expected_kind": "exit",
            "expected_value": 0,
            "expected_stdout_prefix": expectation["stable_stdout_prefix"],
            "validation_status": "validated" if expectation.get("verified") else "unvalidated",
            "feature_coverage": expectation.get("maturity", "unspecified"),
        })
        for index, negative in enumerate(expectation.get("negative_cases", []), start=1):
            negative_source = safe_file(negative["path"])
            if negative_source.parent != check_dir:
                raise SystemExit(f"Negative probe is outside its check directory: {negative['path']}")
            negative_count += 1
            check_cases.append({
                "suite": "checks",
                "case_id": f"{name}/negative-{index}",
                "name": f"{name}-negative-{index}",
                "source": negative_source,
                "source_sha256": fingerprint(negative_source),
                "expected_kind": "compile_reject",
                "expected_value": None,
                "validation_status": "unvalidated",
                "feature_coverage": "negative-probe",
                "expected_rejection_reason": negative.get("reason", ""),
            })

    actual_dirs = sorted(p.name for p in (ROOT / "checks").iterdir()
                         if p.is_dir() and (p / "expectation.json").is_file())
    if sorted(manifest_names) != actual_dirs:
        raise SystemExit(
            "checks/manifest.json does not exactly enumerate expectation-bearing check directories")

    return legacy_cases + check_cases, {
        "legacy_cases": len(legacy_cases),
        "multifile_checks": len(manifest_names),
        "negative_probes": negative_count,
        "total_cases": len(legacy_cases) + len(check_cases),
    }


def evaluate_case(case, backend, execute, allow_unvalidated, timeout):
    source = case["source"]
    record = {
        "suite": case["suite"],
        "case_id": case["case_id"],
        "name": case["name"],
        "backend": backend,
        "source_sha256": case["source_sha256"],
        "validation_status": case["validation_status"],
        "feature_coverage": case["feature_coverage"],
    }
    if "graph_sha256" in case:
        record["graph_sha256"] = case["graph_sha256"]
        record["graph_files"] = case["graph_files"]

    prefix = "ORES_PARITY_" + ENV_NAMES[backend]
    compile_tpl = os.environ.get(prefix + "_COMPILE")
    run_tpl = os.environ.get(prefix + "_RUN")
    if not execute:
        record["status"] = "not_run"
        return record
    if case["validation_status"] != "validated" and not allow_unvalidated:
        record["status"] = "unvalidated"
        return record
    if not compile_tpl or (case["expected_kind"] == "exit" and not run_tpl):
        record["status"] = "backend_not_configured"
        return record

    with tempfile.TemporaryDirectory(prefix="ores-parity-") as tmp:
        workdir = Path(tmp)
        artifact = workdir / "program"
        compile_result = invoke(command(compile_tpl, source, artifact, workdir),
                                workdir, timeout)
        record["compile"] = compile_result
        if compile_result["status"] != "completed":
            record["status"] = "compile_infrastructure_failure"
        elif case["expected_kind"] == "compile_reject":
            if compile_result["exit_code"] == 0:
                record["status"] = "unexpected_compile_success"
            else:
                record["status"] = "rejection_unclassified"
        elif compile_result["exit_code"] != 0:
            record["status"] = "compile_failure"
        else:
            execution = invoke(command(run_tpl, source, artifact, workdir),
                               workdir, timeout)
            record["run"] = execution
            if execution["status"] != "completed":
                record["status"] = "runtime_infrastructure_failure"
            elif execution["exit_code"] != case["expected_value"]:
                record["status"] = "wrong_result"
            elif "expected_stdout_prefix" in case:
                stdout = bytes.fromhex(execution["stdout_hex"])
                expected = case["expected_stdout_prefix"].encode()
                if not stdout.startswith(expected):
                    record["status"] = "wrong_stdout"
                elif case["validation_status"] == "validated":
                    record["status"] = "pass"
                else:
                    record["status"] = "exploratory_match_not_parity_pass"
            elif case["validation_status"] == "validated":
                record["status"] = "pass"
            else:
                record["status"] = "exploratory_match_not_parity_pass"
    return record


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--execute", action="store_true")
    ap.add_argument("--allow-unvalidated", action="store_true",
                    help="Exploratory runs only; never counts toward validated parity")
    ap.add_argument("--output", default="parity-results.json")
    ap.add_argument("--timeout", type=float, default=30.0)
    a = ap.parse_args()
    if a.timeout <= 0:
        ap.error("--timeout must be positive")

    cases, corpus = load_cases()
    records = [evaluate_case(c, backend, a.execute, a.allow_unvalidated, a.timeout)
               for c in cases for backend in BACKENDS]
    result = {
        "schema_version": 2,
        "executed": a.execute,
        "corpus": corpus,
        "cases": len(cases),
        "backends": list(BACKENDS),
        "records": records,
        "summary": {},
    }
    for record in records:
        status = record["status"]
        result["summary"][status] = result["summary"].get(status, 0) + 1
    Path(a.output).write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result["summary"], sort_keys=True))
    fatal = {"compile_failure", "wrong_result", "wrong_stdout",
             "unexpected_compile_success", "compile_infrastructure_failure",
             "runtime_infrastructure_failure"}
    return 1 if any(r["status"] in fatal for r in records) else 0


if __name__ == "__main__":
    raise SystemExit(main())
