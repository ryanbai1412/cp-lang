# Matched C++ for the dry-run perf set

`bench/v03_perf_bench.py --suite dryrun` times each `research/dryrun/v03/<name>.cp`
against `<name>.cpp` here. Each file is `research/dryrun/<name>_ref.cpp` (the stress
reference, owned by idioms) with the same algorithm and data layout, rewritten only to
use the I/O hand-written contest C++ uses (`ios::sync_with_stdio(false)`, untied `cin`,
`'\n'`). Unsynced `cin` and `scanf` made the references 2-17x slower than the
emitted C++ on input alone, which measured I/O, not the language.

`p06_labyrinth.cpp` also prints the path, in the stdlib `bfs` order (`adj4`: up, right,
down, left), so its output is token-identical to the port's.

A problem without a file here falls back to its `_ref.cpp`.
