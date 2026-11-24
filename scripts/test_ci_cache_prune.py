import os
import sys
import subprocess
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ci_cache_prune  # noqa: E402
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

    def test_a_job_prunes_only_the_family_of_the_key_it_saved(self):
        old3, new3 = cache("acceptance-3-45444161207", "1"), cache("acceptance-3-45445387452", "2")
        old13, new13 = cache("acceptance-13-45444161207", "1"), cache("acceptance-13-45445387452", "2")
        self.assertEqual(stale([old3, new3, old13, new13], ["acceptance-3-45445387452"]), [old3])


    def test_a_cache_another_prune_deleted_first_is_not_a_failure(self):
        old = dict(cache("acceptance-3-45444161207", "1"), id=7, size_in_bytes=1)
        caches = {"actions_caches": [old, dict(cache("acceptance-3-45445387452", "2"), id=8, size_in_bytes=1)]}

        def gh(*args):
            if "DELETE" not in args:
                return caches
            raise subprocess.CalledProcessError(1, "gh", stderr=stderr)
        with mock.patch.object(ci_cache_prune, "gh", gh), \
                mock.patch.dict(os.environ, {"GITHUB_REPOSITORY": "o/r"}), \
                mock.patch.object(sys, "argv", ["prune", "--family-of", "acceptance-3-45445387452"]):
            stderr = "gh: Not Found (HTTP 404)"
            ci_cache_prune.main()
            stderr = "gh: Resource not accessible by integration (HTTP 403)"
            with self.assertRaises(subprocess.CalledProcessError):
                ci_cache_prune.main()


if __name__ == "__main__":
    unittest.main()
