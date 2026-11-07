import json
import pathlib
import re
import unittest

from jsonschema import Draft202012Validator, FormatChecker

ROOT = pathlib.Path(__file__).resolve().parents[1] / 'contracts' / 'v1'
SCHEMA = json.loads((ROOT / 'conformance-evidence.schema.json').read_text())
TSP = (ROOT / 'conformance-evidence.tsp').read_text()
WIT = (ROOT / 'conformance-evidence.wit').read_text()
BASE = {'version': 1, 'backend': 'llvm', 'feature_id': 'actor.ownership', 'source_sha': 'a' * 40, 'case_set_sha256': 'b' * 64}
RAN = {**BASE, 'execution_target': 'guest', 'run_url': 'https://github.com/benefactor-cc/ci/actions/runs/42', 'executed_cases': 4, 'failed_cases': 0}
GOOD = [
    {**RAN, 'status': 'passed'},
    {**RAN, 'status': 'failed', 'failed_cases': 1, 'reason': 'nonzero guest exit'},
    {**BASE, 'status': 'unsupported', 'reason': 'target does not implement the ABI'},
    {**BASE, 'status': 'not_executed', 'reason': 'no runner allocated'},
]
BAD = [
    {**GOOD[0], 'status': 'pending'},
    {**GOOD[0], 'version': 2},
    {**GOOD[0], 'backend': 'unknown'},
    {**GOOD[0], 'source_sha': 'a' * 39},
    {**GOOD[0], 'case_set_sha256': 'z' * 64},
    {**GOOD[0], 'run_url': 'http://github.com/a/b/actions/runs/42'},
    {**GOOD[0], 'run_url': 'https://example.org/run/42'},
    {**GOOD[0], 'execution_target': 'simulated'},
    {**GOOD[0], 'executed_cases': 0},
    {**GOOD[0], 'failed_cases': 1},
    {**GOOD[0], 'reason': 'unaudited'},
    {**GOOD[1], 'failed_cases': 0},
    {**GOOD[2], 'run_url': GOOD[0]['run_url']},
    {**GOOD[3], 'executed_cases': 4},
    {**GOOD[3], 'reason': ''},
    {**GOOD[3], 'unexpected': True},
    {**GOOD[3], 'status': 'passed'},
]

class ContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        Draft202012Validator.check_schema(SCHEMA)
        cls.validator = Draft202012Validator(SCHEMA, format_checker=FormatChecker())

    def test_valid_statuses(self):
        for data in GOOD:
            with self.subTest(status=data['status']):
                self.assertTrue(self.validator.is_valid(data), list(self.validator.iter_errors(data)))

    def test_fail_closed_negative_vectors(self):
        for data in BAD:
            with self.subTest(data=data):
                self.assertFalse(self.validator.is_valid(data), data)

    def test_status_enum_lockstep(self):
        statuses = set(SCHEMA['properties']['status']['enum'])
        tsp = set(re.findall(r'status:\s*"([a-z_]+)";', TSP))
        wit = re.search(r'enum\s+evidence-status\s*\{([^}]*)\}', WIT)
        self.assertIsNotNone(wit)
        wit_set = {a.strip().replace('-', '_') for a in wit.group(1).split(',') if a.strip()}
        self.assertEqual(statuses, {'passed', 'failed', 'unsupported', 'not_executed'})
        self.assertEqual(statuses, tsp)
        self.assertEqual(statuses, wit_set)
        self.assertEqual(statuses, {arm['properties']['status']['const'] for arm in SCHEMA['oneOf']})

    def test_backends_and_properties_lockstep(self):
        backend = set(SCHEMA['properties']['backend']['enum'])
        tsp = re.search(r'alias\s+Backend\s*=\s*([^;]+);', TSP)
        wit = re.search(r'enum\s+backend\s*\{([^}]*)\}', WIT)
        self.assertIsNotNone(tsp)
        self.assertIsNotNone(wit)
        self.assertEqual(backend, set(re.findall(r'"([a-z]+)"', tsp.group(1))))
        self.assertEqual(backend, {a.strip() for a in wit.group(1).split(',') if a.strip()})
        rec = re.search(r'record\s+conformance-evidence\s*\{([^}]*)\}', WIT)
        self.assertIsNotNone(rec)
        fields = set(re.findall(r'([a-z][a-z0-9-]*):', rec.group(1)))
        self.assertEqual(fields, {a.replace('_', '-') for a in SCHEMA['properties']})

    def test_draft_closed_and_versioned(self):
        self.assertEqual(SCHEMA['$schema'], 'https://json-schema.org/draft/2020-12/schema')
        self.assertIs(SCHEMA['additionalProperties'], False)
        self.assertEqual(SCHEMA['properties']['version']['const'], 1)
        self.assertIn('package atto:conformance@1.0.0;', WIT)

if __name__ == '__main__':
    unittest.main()
