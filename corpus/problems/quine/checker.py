#!/usr/bin/env python3
"""Checker for DMOJ quine: the right output is the program's own source, so the reference's answer
(the accepted C++ printing itself) says nothing about a port. Accept output equal to this
package's port.cp, up to a final newline; with no port yet, any non-empty output."""
import sys
from pathlib import Path


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    out = Path(sys.argv[2]).read_text(encoding="utf-8", errors="replace")
    port = Path(__file__).resolve().parent / "port.cp"
    if port.exists():
        if out.rstrip("\n") != port.read_text(encoding="utf-8").rstrip("\n"):
            fail("the output is not port.cp's source")
    elif not out.strip():
        fail("the output is empty")


if __name__ == "__main__":
    main()
