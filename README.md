# CP Lang

A language for competitive programming that compiles to one clean C++17
file you can submit to any judge. The command is `cpl` (a working name)
and source files end in `.cp`.

The language is locked at **0.2.2**. The whole surface fits on
[one page](docs/one-page-cheat-sheet.md): 34 reserved keywords (30 for
contestants, 4 for library authors), 4 numeric types and 7 auto-derived
core traits.

## Before and after

Fewest coins that sum to `x`, or `-1` when no combination does. In C++17
the unreachable state is a sentinel that every transition has to guard:

```cpp
#include <bits/stdc++.h>
using namespace std;

const long long INF = 1e18;

int main() {
    int n, x;
    cin >> n >> x;
    vector<int> coins(n);
    for (auto& c : coins) cin >> c;
    vector<long long> dp(x + 1, INF);
    dp[0] = 0;
    for (int s = 1; s <= x; s++)
        for (int c : coins)
            if (c <= s && dp[s - c] != INF)
                dp[s] = min(dp[s], dp[s - c] + 1);
    cout << (dp[x] == INF ? -1 : dp[x]) << "\n";
}
```

In CP Lang 0.2.2 the unreachable state is `none`, and `?Int` does the
bookkeeping:

```cp
let (n, x): (1..=100, 1..=1e6) = read()
let coins: [0..n]Int = read()

var dp: [0..=x]?Int = none
dp[0] = 0
for s in 1..=x:
    for c in coins:
        if c <= s: dp[s] min= dp[s - c] + 1
print(dp[x] ?? -1)
```

`dp[s - c] + 1` is `none` when `dp[s - c]` is, and `min=` skips `none`, so
there is no `INF` to pick and no guard to forget. The bounds in the first
line are the statement's constraints: they size the array and check
the input. It runs today: save it as `coins.cp` and `cpl run coins.cp`
prints `3` for the input `3 11` / `1 5 7`.

## What 0.2.2 is

- **One index mechanism.** `Bool`, enums, `?T`, `lo..=hi`, tuples, records
  and `Subset<D>` all implement `Index`, so `[V]Int`, `[(V, Bool)]Int` and
  `[Subset<D>]Int` are flat arrays. A `domain V = 1..=n` gives typed
  indices that cannot be mixed up with another domain's.
- **Lexical `lends`.** `seg.at(i) += 1` and `swap(mut g.row(a), mut g.row(b))`
  borrow a place for one expression. Borrows are never stored or returned,
  so there are no lifetime annotations.
- **Tropical Optionals.** `?Int` and `?F64` replace `INF` and `-1`: `+`
  and `-` absorb `none`, `min`, `max`, `min=` and `max=` skip it. They are
  niche-packed and take no extra memory.
- **`@memo`.** Write the recurrence as a function. The compiler uses a
  flat table when the parameters are small indices and a salted hash table
  otherwise.
- **`@checker` and `@interactor`.** A validator for multi-answer problems
  and a local judge for interactive ones sit in the same `.cp` file as
  the solution, next to `@brute` and `@gen` for stress testing.
- **7 core traits**, all derived automatically: `Index`, `Seq`, `Eq`,
  `Ord`, `Hash`, `Show`, `Read`. Library authors get two more, `Indexed`
  and `Iterable`.

The reasoning behind each choice is in
[`research/language-architecture-v2.2.md`](research/language-architecture-v2.2.md)
and [`decisions.md`](decisions.md).

## Measured on real submissions

[`bench/corpus-500-report.md`](bench/corpus-500-report.md) tracks ports of
Ryan's accepted contest submissions:

- **573 unique problems** are ported, out of 747 with an accepted
  submission in the corpus.
- **477 of them** have a port that a harness differentially checked
  against the original C++.
- Those 477 ports are **66.4% shorter** than the raw originals and
  **48.4% shorter** than the originals with the template, I/O driver and
  standard data structures stripped out. Over all 573 the figures are
  66.1% and 47.9%.
- 27 ports are no shorter than their stripped original, and 62 hard
  problems are still unported.

## Quickstart

Build the Rust toolchain and run a program in the debug interpreter:

```text
$ cargo build -p cpl-cli
$ printf '3\n1 4 6\n' | target/debug/cpl run site/examples/optional.cp
6
$ target/debug/cpl build site/examples/optional.cp -o -    # the C++17 to submit
$ target/debug/cpl run site/examples/optional.cp --release  # run that C++, built with g++ -O2
$ target/debug/cpl build site/examples/optional.cp --checked # also build it with sanitizers
$ target/debug/cpl --help                                   # check, fmt, lint, test, stress, lsp, ...
```

Run the standard library and sample library tests in the Python
prototype, or compile a port with it:

```text
$ python3 proto/cp.py test stdlib examples/local-lib
$ python3 proto/cp.py build proto/ports/aac4p2.cp            # writes proto/ports/aac4p2.cpp
```

Install the VS Code extension (highlighting, snippets and the `cpl lsp`
client):

```text
$ cd editors/vscode && npm install && npm run package
$ code --install-extension cp-lang.vsix
```

Build the offline course, reference and playground as one HTML file, then
open it straight from disk. It needs no server and no network; the
compiler runs in WebAssembly:

```text
$ python3 -m venv site/.venv && site/.venv/bin/pip install -r site/requirements.txt
$ rustup target add wasm32-unknown-unknown
$ site/.venv/bin/python site/bundle.py                       # writes dist/cp-lang.html
```

## Status

0.2.2 is a locked specification, and the two implementations are still
catching up to it. The Rust compiler (`crates/`) checks, runs and builds a
growing subset; the playground runs that same compiler. The Python
prototype (`proto/`) covers more of the library and builds the corpus
ports, but predates some 0.2.2 spellings. The example above runs in the
Rust compiler, in the interpreter and as emitted C++.
The docs site labels every snippet as either runnable or specification.

## Repository map

- `docs/one-page-cheat-sheet.md`: the 0.2.2 language on one page.
- `decisions.md`: every language decision, its status and its reason.
- `plan/`: what we're building. Start with `plan/agent-context.md`, then
  `plan/README.md` and `plan/00-vision.md`.
- `research/`: the evidence behind the decisions (corpus mining, audits
  and C++ benchmarks).
- `Cargo.toml`, `crates/`: the Rust compiler and tools.
  `crates/README.md` says which crate owns what.
- `proto/`: the Python prototype compiler and the corpus ports
  (`proto/ports/`). Kept for trying design changes quickly.
- `stdlib/`: the standard library, written in CP Lang, with its tests.
  `stdlib/README.md` is the tour.
- `examples/local-lib/`: a sample personal contest library built on it.
- `editors/vscode/`: the VS Code extension.
- `site/`: the interactive course, reference and offline playground. See
  [the site README](site/README.md).
- `bench/`, `tools/bench/`: the corpus registry and the harness that
  measures every port against its original.
- `corpus/`, `subs/`: Ryan's accepted submissions, used for ports,
  benchmarks and design mining.
- `archive/`: superseded design docs.

## Contributing

Work goes straight to `main`; keep it green. There is no hosted CI, so
the checks run on your machine. Turn on the pre-commit hook once per
clone:

```text
$ scripts/install-hooks.sh            # git commit now runs scripts/check.sh --fast
```

`scripts/check.sh --fast` takes about 2 minutes: it refuses staged
`dist/` output and secrets, then runs clippy with warnings as errors,
`cargo test --workspace` and the library tests. Before pushing a change
to the prototype, the WebAssembly build or the VS Code extension, run the
slow suites too:

```text
$ scripts/check.sh --full             # adds the wasm smoke test, the VS Code tests and proto/run_all.sh
```
