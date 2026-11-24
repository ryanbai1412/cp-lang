"""scripts/port-check.sh fails a port on a `cpl check` error, never on a warning."""
import os
import stat
import subprocess
import tempfile
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WARNING = ("warning[L0608]: `s.first` does work\\n  --> corpus/problems/abc480f/port.cp:36:20\\n"
           "corpus/problems/abc480f/port.cp: 2 warnings\\n")
ERROR = "error[E0608]: `out` moves here\\n  --> corpus/problems/abc480f/port.cp:9:5\\n"


def port_check(output):
    with tempfile.TemporaryDirectory() as d:
        cpl = os.path.join(d, "cpl")
        with open(cpl, "w") as f:
            f.write(f'#!/bin/sh\nprintf "{output}" >&2\n')
        os.chmod(cpl, stat.S_IRWXU)
        return subprocess.run(["scripts/port-check.sh", cpl], cwd=ROOT, capture_output=True, text=True)


class PortCheckTest(unittest.TestCase):
    def test_a_warning_never_fails_a_port(self):
        r = port_check(WARNING)
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_an_error_fails_the_port_it_names(self):
        r = port_check(WARNING + ERROR)
        self.assertEqual(r.returncode, 1)
        self.assertIn("  abc480f", r.stderr)

    def test_a_failing_port_shows_its_errors(self):
        r = port_check(ERROR)
        self.assertEqual(r.returncode, 1)
        self.assertIn("error[E0608]", r.stderr)
        self.assertIn("port.cp:9:5", r.stderr)


if __name__ == "__main__":
    unittest.main()
