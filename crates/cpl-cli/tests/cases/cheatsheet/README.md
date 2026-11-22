# Cheat-sheet parity cases

`../test_cheatsheet_parity.py` keeps `docs/one-page-cheat-sheet.md` and the prototype compiler in step.

- `sN_name.cp` (+ `.in`, `.out`): a snippet of cheat-sheet section N that the prototype builds, compiles under
  ASan + UBSan with `-DLOCAL`, and runs to exactly `.out`.
- `gaps/sN_name.cp`: a cheat-sheet construct the prototype does not implement yet. The test requires it to
  still fail; when one starts passing, move it up here with its `.out`.
- `COVERAGE.tsv`: every content line of the cheat sheet (by hash of its text) and the cases or gaps that
  exercise it. A changed or new cheat-sheet line fails the test until it is mapped:
  `python3 tests/test_cheatsheet_parity.py --update` rewrites the file and proposes cases for new lines.

Run from `proto/`: `python3 tests/test_cheatsheet_parity.py`.
