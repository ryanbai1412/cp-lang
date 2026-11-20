"""Build checks for the site trace debugger. Run: python site/test_trace.py"""
import json
from pathlib import Path
import re
import sys
import unittest

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))
sys.path.insert(0, str(ROOT.parent / "tools/trace"))

import build  # noqa: E402
from emit import embed_payload, render_html  # noqa: E402


def payload(page, script_id):
    match = re.search(rf'<script id="{script_id}" type="application/json">(.*?)</script>', page, re.S)
    return json.loads(match.group(1))


class TracePage(unittest.TestCase):
    page = build.trace_html()

    def test_inlines_assets_and_leaves_no_markers(self):
        self.assertNotIn("/*@trace-", self.page)
        self.assertIn("function createPlayer", self.page)
        self.assertIn("--mono:", self.page)

    def test_is_offline_and_fetches_nothing(self):
        self.assertIsNone(re.search(r'<(?:script|link|img)[^>]+(?:src|href)=', self.page))
        for banned in ["@font-face", "@import", "data:font", "fetch(", "XMLHttpRequest"]:
            self.assertNotIn(banned, self.page)

    def test_embeds_every_sample_once(self):
        samples = payload(self.page, "trace-samples")
        self.assertEqual([s["title"] for s in samples],
                         [json.loads((ROOT.parent / f"tools/trace/samples/{n}.json").read_text())["title"] for n in build.TRACE_SAMPLES])
        self.assertIsNone(payload(self.page, "trace-data"))
        self.assertNotIn("\u2028", self.page)

    def test_emitter_contract_still_replaces_both_payloads(self):
        trace = json.loads((ROOT.parent / "tools/trace/samples/knapsack.json").read_text())
        trace["title"] = "</script><b>x</b>\u2028"
        page = render_html(trace, self.page)
        self.assertEqual(payload(page, "trace-data")["title"], trace["title"])
        self.assertEqual(payload(page, "trace-samples"), [])
        self.assertEqual(page.count("</script><b>"), 0)
        self.assertEqual(embed_payload(page, "trace-samples", [trace]).count('id="trace-samples"'), 1)

    def test_names_language_version(self):
        self.assertIn("CP Lang 0.2.2", self.page)


if __name__ == "__main__":
    unittest.main()
