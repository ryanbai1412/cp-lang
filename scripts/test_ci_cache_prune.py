import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ci_cache_prune import family, stale  # noqa: E402

H = "d5fc7a38dbf9fccd39b4915c87add4522c7225110de64702a8177fc924996deb"


def cache(key, created, ref="refs/heads/main"):
    return {"key": key, "created_at": created, "ref": ref}


class StaleTest(unittest.TestCase):
    def test_a_family_drops_its_run_id_hour_and_hash(self):
        self.assertEqual(family(f"target-ci-v2-dev-{H}-45394353893"), "target-ci-v2-dev")
        self.assertEqual(family("corpus-audit-cxx-3-2026112412"), "corpus-audit-cxx-3")
        self.assertEqual(family("cxx-Linux-conform:diff@1/3-2026112412"), "cxx-Linux-conform:diff@1/3")
        self.assertEqual(family("v0-rust-gate-Linux-x64-8272a5e9-5e39c8ef"), "v0-rust-gate-Linux-x64-8272a5e9-5e39c8ef")

    def test_only_the_newest_of_each_main_family_is_kept(self):
        old_dev = cache(f"target-ci-v2-dev-{'0' * 64}-45391542115", "1")
        new_dev = cache(f"target-ci-v2-dev-{H}-45397505715", "3")
        mid_dev = cache(f"target-ci-v2-dev-{H}-45394353893", "2")
        clippy = cache(f"target-ci-v2-clippy-{H}-45394353893", "2")
        old_hour = cache("cxx-Linux-fmt-2026112411", "1")
        hour = cache("cxx-Linux-fmt-2026112412", "2")
        fixed = cache("v0-rust-gate-Linux-x64-8272a5e9-5e39c8ef", "1")
        branch = cache("cxx-Linux-fmt-2026112410", "0", ref="refs/heads/dev")
        caches = [old_dev, new_dev, mid_dev, clippy, old_hour, hour, fixed, branch]
        self.assertEqual(stale(caches), [old_dev, mid_dev, old_hour])


if __name__ == "__main__":
    unittest.main()
