"""scripts/retry.py and scripts/retry.sh: the one retry, in Python and shell."""

from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from retry import retry  # noqa: E402

HERE = Path(__file__).parent


class RetryPy(unittest.TestCase):
    def test_a_transient_failure_is_tried_again(self) -> None:
        calls = []

        def call() -> str:
            calls.append(1)
            if len(calls) < 3:
                raise TimeoutError("slow")
            return "ok"

        self.assertEqual(retry(call, transient=lambda e: True, wait=0), "ok")
        self.assertEqual(len(calls), 3)

    def test_the_last_failure_is_raised(self) -> None:
        calls = []

        def call() -> None:
            calls.append(1)
            raise TimeoutError(str(len(calls)))

        with self.assertRaisesRegex(TimeoutError, "^3$"):
            retry(call, transient=lambda e: True, wait=0)

    def test_a_permanent_failure_is_not_tried_again(self) -> None:
        calls = []

        def call() -> None:
            calls.append(1)
            raise ValueError("bad request")

        with self.assertRaises(ValueError):
            retry(call, transient=lambda e: isinstance(e, TimeoutError), wait=0)
        self.assertEqual(len(calls), 1)


class RetrySh(unittest.TestCase):
    def run_sh(self, script: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["bash", "-c", f'. "{HERE}/retry.sh"; RETRY_WAIT=0 {script}'],
                              capture_output=True, text=True)

    def test_a_command_that_passes_on_the_third_try_passes(self) -> None:
        r = self.run_sh('n=0; f() { n=$((n + 1)); [ "$n" = 3 ]; }; retry f && echo "tries=$n"')
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stdout.strip(), "tries=3")
        self.assertIn("try 2 of 3 failed", r.stderr)

    def test_a_command_that_never_passes_fails_naming_it(self) -> None:
        r = self.run_sh("RETRY_TRIES=2 retry false")
        self.assertEqual(r.returncode, 1)
        self.assertIn("gave up after 2 tries: false", r.stderr)


if __name__ == "__main__":
    unittest.main()
