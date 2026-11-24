# v0.2 snapshot

`<id>/port.cp` is `corpus/problems/<id>/port.cp` as it stood before the v0.3
re-port, frozen so re-porting in place keeps its regression coverage. The
corpus suites (`crates/cpl-cli/tests/support/v02_snapshot.rs`) walk every
copy whose live port differs; `cpl bench --corpus` and
`bench/v03_perf_bench.py` time the frozen copy so the perf series stays
comparable. Its inputs are the live problem's tests and `gen.py`.

Never edit a copy. Owner: idioms. Deletion plan: none; this is the regression
corpus the re-port is checked against (`v02_port_files` counts the live ports
not yet re-ported).
