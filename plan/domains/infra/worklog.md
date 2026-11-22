# infra worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Note from lint-fmt: added the `cpl fmt is idempotent and keeps meaning on every .cp file` step to `scripts/check.sh` (runs `scripts/fmt_check.py` directly, so `direct_python` stays 13, beside nextest, cached as stage `fmt_corpus` in `scripts/check_cache.py`; about 6 s on 8 cores, cold), plus `scripts/test_fmt_check.py` in the script unit tests. Revert that commit to drop it.
- Lane opened.
