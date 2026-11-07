import json
import pathlib
import re
import unittest
from contracts.strict_json import load_and_validate, load_conformance_evidence, MAX_WIRE_BYTES

ROOT = pathlib.Path(__file__).resolve().parents[1] / "contracts" / "v1"
SCHEMA = json.loads((ROOT / "conformance-evidence.schema.json").read_text())
WIT = (ROOT / "conformance-evidence.wit").read_text()
TSP = (ROOT / "conformance-evidence.tsp").read_text()
BASE = {"version": 1, "backend": "llvm", "feature_id": "actor.ownership",
        "source_sha": "a"*40, "case_set_sha256": "b"*64}
RAN = {**BASE, "status": "passed", "execution_target": "guest",
       "run_url": "https://github.com/benefactor-cc/ci/actions/runs/42",
       "executed_cases": 5, "failed_cases": 0}
FAIL = {**RAN, "status": "failed", "failed_cases": 2, "reason": "exit mismatch"}
SKIP = {**BASE, "status": "not_executed", "reason": "runner unavailable"}
UNSUPPORTED = {**BASE, "status": "unsupported", "reason": "backend no implementation"}

def wit_fields(text):
    m = re.search(r"record\s+conformance-evidence\s*\{([^}]*)\}", text)
    if not m:
        raise AssertionError("missing WIT evidence record")
    return dict((k,v.strip()) for k,v in re.findall(r"(?m)^\s*([a-z-]+):\s*([^,\n]+)",m.group(1)))

def tsp_fields(text):
    result = {}
    for name,typ in re.findall(r"(?m)^\s*([a-z][a-z0-9_]*)\s*:\s*([^;\n]+);", text):
        result.setdefault(name,set()).add(typ.strip())
    return result

WIT_EXPECTED = {
    "version": "u32", "backend": "backend", "feature-id": "string",
    "source-sha": "string", "case-set-sha256": "string", "status": "evidence-status",
    "execution-target": "option<execution-target>", "run-url": "option<string>",
    "executed-cases": "option<u32>", "failed-cases": "option<u32>", "reason": "option<string>",
}
TSP_EXPECTED = {
    "version": {"1"}, "backend": {"Backend"}, "feature_id": {"string"},
    "source_sha": {"string"}, "case_set_sha256": {"string"},
    "status": {'"passed"','"failed"','"unsupported"','"not_executed"'},
    "execution_target": {"ExecutionTarget"}, "run_url": {"url"},
    "executed_cases": {"uint32"}, "failed_cases": {"0","uint32"}, "reason": {"string"},
}

class HardeningTests(unittest.TestCase):
    def test_structural_projection(self):
        self.assertEqual(wit_fields(WIT), WIT_EXPECTED)
        self.assertEqual(tsp_fields(TSP), TSP_EXPECTED)
        self.assertEqual(set(SCHEMA["properties"]), {k.replace("-", "_") for k in WIT_EXPECTED})

    def test_projection_mutation_detection(self):
        self.assertNotEqual(wit_fields(WIT.replace("executed-cases: option<u32>", "executed-cases: u64")), WIT_EXPECTED)
        self.assertNotEqual(tsp_fields(TSP.replace("source_sha: string;", "source_sha: uint32;")), TSP_EXPECTED)

    def test_all_statuses_parse(self):
        for payload in [RAN, FAIL, SKIP, UNSUPPORTED]:
            with self.subTest(payload=payload):
                self.assertEqual(load_conformance_evidence(json.dumps(payload), SCHEMA), payload)

    def test_semantic_count_invariant(self):
        invalid = {**FAIL, "executed_cases": 1, "failed_cases": 2}
        self.assertEqual(load_and_validate(json.dumps(invalid), SCHEMA), invalid)
        with self.assertRaises(ValueError):
            load_conformance_evidence(json.dumps(invalid), SCHEMA)

    def test_type_confusion_matrix(self):
        text_types = [None, True, 123, 1.5, [], {}, ""]
        integer_types = [None, True, False, "1", 1.5, [], {}]
        groups = [
            (RAN,"backend",[None,True,123,[],{},"LLVM","unknown"]),
            (RAN,"feature_id",text_types),
            (RAN,"source_sha",text_types+["a"*40+"\n"]),
            (RAN,"case_set_sha256",text_types+["b"*64+"\n"]),
            (RAN,"status",[None,True,123,[],{},"pending","skipped"]),
            (RAN,"execution_target",[None,True,123,[],{},"simulated",""]),
            (RAN,"run_url",text_types+[RAN["run_url"]+"\n"]),
            (RAN,"executed_cases",integer_types+[0]),
            (RAN,"failed_cases",integer_types+[1]),
            (FAIL,"reason",text_types),
            (RAN,"version",[None,True,False,-1,2,"1",1.0]),
        ]
        count=0
        for base,name,variants in groups:
            for replacement in variants:
                with self.subTest(name=name,value=repr(replacement)):
                    with self.assertRaises(ValueError):
                        load_conformance_evidence(json.dumps({**base,name:replacement}), SCHEMA)
                count+=1
        self.assertGreaterEqual(count,75)

    def test_boundary_rejections(self):
        invalid=[
            {**BASE,"status":"passed"}, {**RAN,"failed_cases":1},
            {**RAN,"source_sha":"a"*39},{**RAN,"source_sha":"a"*40+"\n"},
            {**RAN,"case_set_sha256":"b"*65},
            {**RAN,"feature_id":"feature\n"}, {**RAN,"run_url":RAN["run_url"]+"\n"},
            {**RAN,"run_url":"http://github.com/a/b/actions/runs/42"},
            {**FAIL,"failed_cases":0}, {**SKIP,"run_url":RAN["run_url"]},
            {**UNSUPPORTED,"executed_cases":0}, {**SKIP,"reason":""},
            {**SKIP,"unknown":1}, {**RAN,"reason":"claim"},
        ]
        for payload in invalid:
            with self.subTest(payload=str(payload)[:120]):
                with self.assertRaises(ValueError):
                    load_conformance_evidence(json.dumps(payload), SCHEMA)

    def test_ambiguous_json_fail_closed(self):
        raw=json.dumps(RAN)
        for key in RAN:
            value=raw[:-1]+", "+json.dumps(key)+": "+json.dumps(RAN[key])+"}"
            with self.subTest(key=key):
                with self.assertRaises(ValueError):
                    load_conformance_evidence(value, SCHEMA)
        for value in [b"\xff", b"{", b"null", b"{}", b"[]", b" "*(MAX_WIRE_BYTES+1),
                      json.dumps({**RAN,"version":1.0}),
                      json.dumps({**RAN,"feature_id":"\ud800"}),
                      raw[:-1]+', "unknown": NaN}']:
            with self.subTest(value=repr(value[:70])):
                with self.assertRaises((ValueError,UnicodeError)):
                    load_conformance_evidence(value, SCHEMA)

if __name__ == "__main__":
    unittest.main()
