"""The one retry in Python scripts, beside scripts/retry.sh
(handbook/guidelines/ci-checklist.md ci/R34): a network call tried at most
`tries` times, waiting `wait`, then twice that, ..., between tries. Each
failed try is named on stderr; the last one, or one `transient` rejects, is
raised. debt-ratchet's ci_retries counts every retry outside these two files.
"""

from __future__ import annotations

import sys
import time
from typing import Callable, TypeVar

T = TypeVar("T")


def retry(call: Callable[[], T], *, transient: Callable[[Exception], bool],
          tries: int = 3, wait: float = 2.0) -> T:
    n = 1
    while True:
        try:
            return call()
        except Exception as exc:
            if n >= tries or not transient(exc):
                raise
            print(f"retry: try {n} of {tries} failed: {exc}", file=sys.stderr)
            time.sleep(wait * 2 ** (n - 1))
            n += 1
