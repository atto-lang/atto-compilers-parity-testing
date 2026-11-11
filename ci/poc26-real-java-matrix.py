#!/usr/bin/env python3
"""Run the actual pinned Java/GraalVM launcher, never pretend compiler negatives are host assertions."""
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = json.loads((ROOT / "ci/poc26-java-compiler-mirror-manifest.json").read_text())
GUEST = [
    ("01-factory-method", "src/patterns/01-factory-method"),
    ("39-actor-model", "src/patterns/39-actor-model"),
    ("71-idempotent-consumer", "src/patterns/71-idempotent-consumer"),
]
NEGATIVE = [
    ("immutable-reassignment", "src/validation/atto-tip/negative/type-first-reassignment.atto",
     r"(?i)(?:immutable|reassign|cannot assign|assignment|read.only|not mutable)"),
    ("removed-modulo", "src/validation/atto-tip/negative/removed-modulo.atto",
     r"(?i)(?:modulo|operator|%|unexpected|invalid|lexer|parse)"),
    ("removed-caret", "compiler/contracts/atto-scalar-v1/negative/removed-caret.atto",
     r"(?i)(?:caret|operator|\^|unexpected|invalid|lexer|parse)"),
]
cp_path = ROOT / "compiler/target/classpath.txt"
classpath = str(ROOT / "compiler/target/classes") + os.pathsep + cp_path.read_text().strip()
if not cp_path.read_text().strip():
    raise SystemExit("missing resolved GraalVM Maven dependency classpath")
def file_digest(file):
    return hashlib.sha256(file.read_bytes()).hexdigest()
jar = ROOT / "compiler/target/oreslang-source-0.1.0-SNAPSHOT.jar"
if not jar.is_file():
    raise SystemExit("Maven build artifact absent")
report = {
    "source_commit": MANIFEST["poc_commit"],
    "source_tree": MANIFEST["poc_tree"],
    "compiler_commit": MANIFEST["compiler_commit"],
    "compiler_tree": MANIFEST["compiler_tree"],
    "compiled_jar_sha256": file_digest(jar),
    "github_run_id": os.getenv("GITHUB_RUN_ID", "unknown"),
    "github_run_attempt": os.getenv("GITHUB_RUN_ATTEMPT", "unknown"),
    "tests": [],
}
def execute(name, mode, source):
    argv = ["java", "-Xmx1024m", "--sun-misc-unsafe-memory-access=allow", "--enable-native-access=ALL-UNNAMED", "-Dpolyglot.engine.WarnInterpreterOnly=false", "-cp",
            classpath, "dev.oreslang.launcher.OresMain"]
    if mode == "check":
        argv.append("--check")
    argv.append(str((ROOT / source).resolve()))
    child = subprocess.Popen(argv, cwd=ROOT / "compiler", stdin=subprocess.DEVNULL,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    try:
        out, err = child.communicate(timeout=50)
        timed_out = False
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, signal.SIGKILL)
        out, err = child.communicate()
        timed_out = True
    return {
        "name": name, "mode": mode, "path": source, "exit": child.returncode,
        "timeout": timed_out,
        "stdout": out[:65536].decode("utf-8", "replace"),
        "stderr": err[:65536].decode("utf-8", "replace"),
        "truncated": len(out) > 65536 or len(err) > 65536,
        "sha256": file_digest(ROOT / source),
    }
failed = 0
for name, folder in GUEST:
    source = folder + "/example.atto"
    oracle = (ROOT / folder / "expected.stdout").read_text(encoding="utf-8")
    checked = execute(name, "check", source)
    checked["pass"] = checked["exit"] == 0 and not checked["timeout"] and not checked["truncated"]
    report["tests"].append(checked)
    print("COMPILER_CHECK", name, "PASS" if checked["pass"] else "FAIL",
          "exit", checked["exit"], "stderr", repr(checked["stderr"][-300:]), flush=True)
    if not checked["pass"]:
        failed += 1
        continue
    ran = execute(name, "guest", source)
    ran["pass"] = (ran["exit"] == 0 and not ran["timeout"] and not ran["truncated"]
                   and ran["stdout"] == oracle and not ran["stderr"])
    report["tests"].append(ran)
    print("GRAALVM_GUEST", name, "PASS" if ran["pass"] else "FAIL",
          "exit", ran["exit"], "out", repr(ran["stdout"][-300:]),
          "stderr", repr(ran["stderr"][-300:]), flush=True)
    if not ran["pass"]:
        failed += 1
for name, source, diagnostic in NEGATIVE:
    invalid = execute(name, "check", source)
    text = invalid["stderr"] + invalid["stdout"]
    invalid["pass"] = (invalid["exit"] not in (0, None) and not invalid["timeout"]
                       and not invalid["truncated"]
                       and bool(re.search(diagnostic, text))
                       and "not a file" not in text.lower()
                       and "noclassdeffounderror" not in text.lower())
    report["tests"].append(invalid)
    print("COMPILER_NEGATIVE", name, "PASS" if invalid["pass"] else "FAIL",
          "exit", invalid["exit"], "diagnostic", repr(text[-450:]), flush=True)
    if not invalid["pass"]:
        failed += 1
path = ROOT / "ci/poc26-real-java-results.json"
path.write_text(json.dumps(report, indent=2) + "\n")
print("COMPILER_REAL_MATRIX", len(report["tests"]), "executed;", failed, "failed;",
      "SOURCE_HEAD", MANIFEST["poc_commit"], "COMPILER_HEAD", MANIFEST["compiler_commit"], flush=True)
sys.exit(1 if failed else 0)
