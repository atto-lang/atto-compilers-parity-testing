#!/usr/bin/env python3
"""Actual Java/GraalVM compiler and guest audit of all 90 PoC patterns + 32 libraries."""
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
POC = json.loads((ROOT / "ci/poc26-source-manifest.json").read_text())
COMPILER = json.loads((ROOT / "ci/poc26-java-compiler-mirror-manifest.json").read_text())
assert POC["source_commit"] == COMPILER["poc_commit"]
assert POC["source_tree"] == COMPILER["poc_tree"]
cases = []
for category, glob in (("pattern", "src/patterns/[0-9][0-9]-*/example.atto"),
                       ("library", "src/libraries/oreslang-*/example.atto")):
    for source in sorted(ROOT.glob(glob)):
        rel = source.relative_to(ROOT).as_posix()
        oracle = source.with_name("expected.stdout")
        expect = oracle.relative_to(ROOT).as_posix()
        cases.append((category, rel, expect))
assert sum(c[0] == "pattern" for c in cases) == 90, "exact 90 pattern admission required"
assert sum(c[0] == "library" for c in cases) == 32, "exact 32 library admission required"
assert len(cases) == 122
def sha1_file(p):
    data = p.read_bytes()
    return hashlib.sha1(f"blob {len(data)}".encode("ascii") + bytes([0]) + data).hexdigest()
for _,src,expected in cases:
    for name in (src,expected):
        path = ROOT / name
        assert path.is_file() and not path.is_symlink(), f"missing or symlinked: {name}"
        assert POC["files"].get(name) == sha1_file(path), f"not source-exact: {name}"
classpath_path=ROOT / "compiler/target/classpath.txt"
assert classpath_path.is_file() and classpath_path.read_text().strip()
classpath=str(ROOT / "compiler/target/classes")+os.pathsep+classpath_path.read_text().strip()
env={k:v for k,v in os.environ.items() if not any(x in k.upper() for x in
     ("TOKEN","PASSWORD","CREDENTIAL","SECRET"))}
report={
  "poc_commit":POC["source_commit"],
  "poc_tree":POC["source_tree"],
  "compiler_commit":COMPILER["compiler_commit"],
  "compiler_tree":COMPILER["compiler_tree"],
  "donor_head":os.popen("git rev-parse HEAD").read().strip(),
  "run_id":os.environ.get("GITHUB_RUN_ID"),
  "attempt":os.environ.get("GITHUB_RUN_ATTEMPT"),
  "scope":"actual Java/GraalVM --check and guest, 90 patterns and 32 library probes",
  "results":[]
}
dest=ROOT/"ci/poc26-all122-real-java-results.json"
def execute(mode,rel):
    argv=["java","-Xmx1024m","--sun-misc-unsafe-memory-access=allow",
          "--enable-native-access=ALL-UNNAMED",
          "-Dpolyglot.engine.WarnInterpreterOnly=false","-cp",classpath,
          "dev.oreslang.launcher.OresMain"]
    if mode=="check":argv.append("--check")
    argv.append(str((ROOT/rel).resolve()))
    p=subprocess.Popen(argv,cwd=ROOT/"compiler",env=env,
        stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    try:
        stdout,stderr=p.communicate(timeout=35)
        timeout=False
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGKILL)
        stdout,stderr=p.communicate()
        timeout=True
    return {"exit":p.returncode,"timeout":timeout,
            "stdout":stdout[:65536].decode("utf-8","replace"),
            "stderr":stderr[:65536].decode("utf-8","replace"),
            "truncated":len(stdout)>65536 or len(stderr)>65536}
totals={"compile_pass":0,"compile_fail":0,"guest_pass":0,"guest_fail":0,"guest_not_executed":0}
for idx,(category,source,oracle) in enumerate(cases,1):
    name=Path(source).parent.name
    expected=(ROOT/oracle).read_text(encoding="utf-8")
    checked=execute("check",source)
    compile_pass=checked["exit"]==0 and not checked["timeout"] and not checked["truncated"]
    totals["compile_pass" if compile_pass else "compile_fail"]+=1
    guest=None
    if compile_pass:
        guest=execute("guest",source)
        guest_pass=(guest["exit"]==0 and not guest["timeout"] and not guest["truncated"]
                    and guest["stdout"]==expected and guest["stderr"]=="")
        totals["guest_pass" if guest_pass else "guest_fail"]+=1
    else:
        guest_pass=False
        totals["guest_not_executed"]+=1
    admission_ok=all(POC["files"][p]==sha1_file(ROOT/p) for p in (source,oracle))
    if not admission_ok:
        compile_pass=False;guest_pass=False
        raise RuntimeError(f"source/oracle changed during execution: {source}")
    result={"category":category,"case":name,"source":source,"oracle":oracle,
            "compile_pass":compile_pass,"compile":checked,
            "guest_pass":guest_pass if guest is not None else None,
            "guest":guest}
    report["results"].append(result)
    report["totals"]=totals
    dest.write_text(json.dumps(report,indent=2)+"\n")
    print(f"REAL_ATTO {idx:03d}/122 {category} {name}: "
          f"compile={'PASS' if compile_pass else 'FAIL'} "
          f"guest={'PASS' if guest_pass else ('FAIL' if guest is not None else 'NOT_EXECUTED')} "
          f"error={repr((checked['stderr'] if not compile_pass else (guest['stderr'] if guest else ''))[:220])}",
          flush=True)
print("REAL_ATTO_122_SUMMARY",json.dumps(totals), "POC_SHA",POC["source_commit"],
      "COMPILER_SHA",COMPILER["compiler_commit"],flush=True)
if totals["compile_pass"]!=122 or totals["guest_pass"]!=122:
    sys.exit(1)
