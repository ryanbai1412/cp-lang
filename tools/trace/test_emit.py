import json
from pathlib import Path
import re
import tempfile
import unittest

from emit import embed_payload, render_html, safe_json
from samples import all_samples

DIRECTORY = Path(__file__).parent


class EmitterTests(unittest.TestCase):
    def test_dangerous_source_round_trips_without_ending_script(self):
        trace = all_samples()[0]
        trace["source"]["code"] = '</script><script>alert("x")</script>\u2028\u2029<!--'
        html = render_html(trace)
        data = re.search(r'<script id="trace-data" type="application/json">(.*?)</script>', html, re.S)[1]
        self.assertNotIn("<", data)
        self.assertNotIn("\u2028", data)
        self.assertNotIn("\u2029", data)
        self.assertEqual(json.loads(data), trace)
        self.assertIn('<script id="trace-samples" type="application/json">[]</script>', html)

    def test_template_markers_must_be_unique(self):
        with self.assertRaisesRegex(ValueError, "exactly one"):
            embed_payload("<html/>", "trace-data", {})
        marker = '<script id="trace-data" type="application/json">null</script>'
        with self.assertRaisesRegex(ValueError, "exactly one"):
            embed_payload(marker + marker, "trace-data", {})

    def test_special_backslashes_are_not_regex_replacement_instructions(self):
        value = {"source": r"\1\n\g<1>", "title": "<>&"}
        self.assertEqual(json.loads(safe_json(value)), value)

    def test_no_nan_or_infinity_in_emitted_json(self):
        for value in [float("nan"), float("inf"), -float("inf")]:
            with self.assertRaises(ValueError):
                safe_json({"value": value})

    def test_rejects_unknown_format(self):
        with self.assertRaisesRegex(ValueError, "schemaVersion"):
            render_html({"schemaVersion": 2})

    def test_sample_generation_is_deterministic(self):
        self.assertEqual(all_samples(), all_samples())
        for name, trace in zip(["knapsack", "dijkstra", "tree", "grid", "array"], all_samples()):
            self.assertEqual(json.loads((DIRECTORY / "samples" / f"{name}.json").read_text()), json.loads(json.dumps(trace)))

    def test_cli_emits_a_standalone_file(self):
        import subprocess
        import sys
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "trace.html"
            subprocess.run([sys.executable, str(DIRECTORY / "emit.py"), str(DIRECTORY / "samples" / "array.json"), str(output)], check=True, capture_output=True)
            html = output.read_text()
            self.assertTrue(html.startswith("<!doctype html>"))
            self.assertIn('id="trace-core"', html)
            self.assertNotRegex(html, r'<(?:script|link)[^>]+(?:src|href)=["\']https?://')


if __name__ == "__main__":
    unittest.main()
