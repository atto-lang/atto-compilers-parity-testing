#!/usr/bin/env python3
"""Fail-closed cross-backend Oreslang parity runner.

Configure each backend independently with ORES_PARITY_<BACKEND>_COMPILE
and ORES_PARITY_<BACKEND>_RUN command templates. Template tokens:
{source} {artifact} {workdir}. Use shlex command arrays, never shell=True.
To actually execute, pass --execute; otherwise run a manifest audit.
A template may be a wrapper script that emits compiler artifacts.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
BACKENDS = ("llvm", "java", "wasm", "dotnet", "js")
ENV_NAMES = {"llvm": "LLVM", "java": "JAVA", "wasm": "WASM",
             "dotnet": "DOTNET", "js": "JS"}

def fingerprint(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

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
    manifest = json.loads((ROOT / "manifest.json").read_text())
    records = []
    for c in manifest["cases"]:
        source = (ROOT / c["path"]).resolve()
        # Prevent path escape from an untrusted manifest.
        if ROOT not in source.parents or not source.is_file():
            raise SystemExit("Invalid source path: " + c["path"])
        for backend in BACKENDS:
            record = {"case_id": c["id"], "backend": backend,
                      "source_sha256": fingerprint(source),
                      "validation_status": c["validation_status"],
                      "feature_coverage": c["feature_coverage"]}
            prefix = "ORES_PARITY_" + ENV_NAMES[backend]
            compile_tpl = os.environ.get(prefix+"_COMPILE")
            run_tpl = os.environ.get(prefix+"_RUN")
            if not a.execute:
                record["status"] = "not_run"
            elif c["validation_status"] != "validated" and not a.allow_unvalidated:
                record["status"] = "unvalidated"
            elif not compile_tpl or (c["expected_kind"] == "exit" and not run_tpl):
                record["status"] = "backend_not_configured"
            else:
                with tempfile.TemporaryDirectory(prefix="ores-parity-") as tmp:
                    workdir = Path(tmp)
                    artifact = workdir / "program"
                    compile_result = invoke(command(compile_tpl, source, artifact, workdir),
                                            workdir, a.timeout)
                    record["compile"] = compile_result
                    if compile_result["status"] != "completed":
                        record["status"] = "compile_infrastructure_failure"
                    elif c["expected_kind"] == "compile_reject":
                        if compile_result["exit_code"] == 0:
                            record["status"] = "unexpected_compile_success"
                        else:
                            record["status"] = "rejection_unclassified"
                    elif compile_result["exit_code"] != 0:
                        record["status"] = "compile_failure"
                    else:
                        execution = invoke(command(run_tpl, source, artifact, workdir),
                                           workdir, a.timeout)
                        record["run"] = execution
                        if execution["status"] != "completed":
                            record["status"] = "runtime_infrastructure_failure"
                        elif execution["exit_code"] != c["expected_value"]:
                            record["status"] = "wrong_result"
                        elif c["validation_status"] == "validated":
                            record["status"] = "pass"
                        else:
                            record["status"] = "exploratory_match_not_parity_pass"
            records.append(record)
    result = {"schema_version": 1, "executed": a.execute,
              "cases": len(manifest["cases"]), "backends": list(BACKENDS),
              "records": records, "summary": {}}
    for record in records:
        s = record["status"]
        result["summary"][s] = result["summary"].get(s, 0) + 1
    Path(a.output).write_text(json.dumps(result, indent=2)+"\n")
    print(json.dumps(result["summary"], sort_keys=True))
    return 1 if any(r["status"] in {"compile_failure", "wrong_result",
               "unexpected_compile_success", "compile_infrastructure_failure",
               "runtime_infrastructure_failure"} for r in records) else 0

if __name__ == "__main__":
    raise SystemExit(main())
