# CP Lang

A language for competitive programming that compiles to one clean C++17
file you can submit to any judge. The command is `cpl` (a working name)
and source files end in `.cp`.

The language is locked at **0.2.2**. The whole surface fits on
[one page](web/content/docs/guides/cheatsheet.mdx): 34 reserved keywords (30 for
contestants, 4 for library authors), 4 numeric types and 7 auto-derived
core traits.

## Where things stand

- [Latest ports](research/PORTS.md): 275 accepted solutions rethought in CP Lang,
  newest first, each with its C++ original, runtime ratio, and friction report
  (contestant tokens median **0.50x** stripped C++; C++ runtime **p50 0.99x**;
  dream gap across 27 `dream/*.cp` ports: **11,825** port vs **7,639** dream
  tokens, median **1.33x**, p90 **2.53x**, ratcheted in `crates/cpl-conform` / `just conform`).
- [Domains](plan/DOMAINS.md): each part of the project, its success
  criteria, plan and worklog.
- [Decisions](decisions.md) and [retros](research/retros/): design calls with
  their reasons, and why the way we work changed.
- [Normative spec](spec/README.md): the v0.3 language, library, tools, and conformance spec.
- Working on the code? Start with [AGENTS.md](AGENTS.md), then
  [REPO.md](REPO.md) for the layout.

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
  and `-` absorb `none`, `min`, `max`, `min=` and `max=` skip it. In the
  emitted C++ a `?Int` is still a 16-byte `std::optional<long long>`;
  packing it into 8 bytes is an open item in
  [the perf tail profile](research/perf/perf-tail-profile-2.md).
- **`@memo`.** Write the recurrence as a function. The compiler uses a
  flat table when the parameters are small indices and a salted hash table
  otherwise.
- **`@check` and `@interactor`.** A validator for multi-answer problems
  and a local judge for interactive ones sit in the same `.cp` file as
  the solution, next to `@brute` and `@gen` for stress testing.
- **7 core traits**, all derived automatically: `Index`, `Seq`, `Eq`,
  `Ord`, `Hash`, `Show`, `Read`. Library authors get two more, `Indexed`
  and `Iterable`.

The reasoning behind each choice is in
[`research/archive/architecture/language-architecture-v2.2.md`](research/archive/architecture/language-architecture-v2.2.md)
and [`decisions.md`](decisions.md).

## Measured on real submissions

[`research/perf/corpus-500-report.md`](research/perf/corpus-500-report.md) tracks ports of
Ryan's accepted contest submissions:

- **579 unique problems** are ported (971 `.cp` files), out of 747 with
  an accepted submission in the corpus.
- **566 of them** have a port that a harness differentially checked
  against the original C++.
- The canonical ports are **65.7% shorter** than the raw originals and
  **47.4% shorter** than the originals with the template, I/O driver and
  standard data structures stripped out (23657 CP Lang lines against
  69041 raw and 45007 stripped).
- 30 ports are no shorter than their stripped original, and 56 hard
  problems are still unported.

## Setup

[mise](https://mise.jdx.dev) pins every tool in `mise.toml` (Rust, Python,
Node, pnpm, uv, just, cargo-nextest). On Linux or macOS, with `g++` on the
path:

```text
$ curl https://mise.run | sh && eval "$(~/.local/bin/mise activate bash)"
$ mise trust && mise install   # the pinned tools
$ pnpm install                 # web/ and editors/vscode/
$ uv sync                      # the Python scripts
$ just hooks                   # pre-commit runs the fast gate
$ just check                   # the fast gate: clippy, tests, corpus, site snippets
```

On macOS some fast-gate tests fail (the emitted C++ leans on libstdc++);
Linux is the reference.

## Quickstart

Build the Rust toolchain and run a program in the debug interpreter:

```text
$ cargo build -p cpl-cli
$ printf '3\n1 4 6\n' | target/debug/cpl run web/content/landing/hero/optional.cp
6
$ target/debug/cpl build web/content/landing/hero/optional.cp -o -    # the C++17 to submit
$ target/debug/cpl run web/content/landing/hero/optional.cp --release  # run that C++, built with g++ -O2
$ target/debug/cpl build web/content/landing/hero/optional.cp --checked # also build it with sanitizers
$ target/debug/cpl --help                                   # check, fmt, lint, test, stress, lsp, ...
```

Compile a corpus
port:

```text
$ target/debug/cpl build corpus/problems/aac4p2/port.cp -o - # the C++17 for one port
```

Install the VS Code extension (highlighting, snippets and the `cpl lsp`
client):

```text
$ cd editors/vscode && npm install && npm run package
$ code --install-extension cp-lang.vsix
```

Neovim, Emacs and Helix use the same server; their setups
are in `editors/README.md`.

Build the website (course, guides, reference and playground) as static
files. The compiler runs in the browser as WebAssembly, so it needs no
backend:

```text
$ rustup target add wasm32-unknown-unknown
$ scripts/build-wasm.sh                                      # writes dist/wasm/
$ pnpm install --frozen-lockfile && pnpm --filter cp-lang-web build   # writes web/out/
```

## Status

0.2.2 is a locked specification, and the Rust compiler (`crates/`) is
its implementation; the playground and the VS Code extension run that
same compiler. It checks, runs and builds the whole standard library and
the sample library. `crates/cpl-cli/tests/corpus.rs` checks the corpus
ports and the prototype's tests; the files it does not pass yet are listed
there as `PENDING` or `NOT_BUILT`, and
[`research/archive/frictions/friction-queue.md`](research/archive/frictions/friction-queue.md) says what each
one waits on. The example above runs in the interpreter and as
emitted C++.
The docs site labels every snippet as either runnable or specification.

## Repository map

- `spec/`: the normative 0.3 language, library, tools, and conformance spec.
- `decisions.md`: every language decision, its status and its reason.
- `plan/`: operating rules (`plan/OPERATING.md`), domains (`plan/DOMAINS.md`, `plan/domains/`),
  architecture (`plan/ARCHITECTURE.md`), and Ryan's notes (`plan/agent-context.md`).
- `research/`: the evidence behind the decisions (corpus mining, audits,
  design history, and C++ benchmarks).
- `Cargo.toml`, `crates/`: the Rust compiler and tools.
  `crates/README.md` says which crate owns what.
- `corpus/`: canonical problem ports (`corpus/problems/<id>/port.cp`, `dream.cp`,
  `meta.json`), shared libraries (`corpus/lib/`), and raw accepted submissions
  (`corpus/submissions/`).
- `research/archive/frictions/friction-queue.md`: every open friction item from the ports,
  clustered, with the next syntax batches.
- `std/`: the standard library, written in CP Lang.
  `std/README.md` is the tour.
- `editors/vscode/`: the VS Code extension.
- `web/`: the website: landing page, course, guides, reference and
  playground. `web/scripts/gate.mjs` checks every program on it.
- `bench/`: the corpus performance harness and results.
- `mise.toml`, `justfile`, `pyproject.toml`, `pnpm-workspace.yaml`: unified
  toolchain pins and task runner (see [Setup](#setup)).

## Contributing

Work goes straight to `main`; keep it green. There is no hosted CI, so
the checks run on your machine. Turn on the pre-commit hook once per
clone:

```text
$ just hooks                          # git commit now runs scripts/check.sh --fast
```

`scripts/check.sh --fast` takes under a minute on a warm build: it
refuses staged `dist/` output and secrets, then runs clippy with warnings
as errors, the workspace tests (with `cargo nextest` when it is
installed), the library tests, `cpl fmt --check stdlib` and the friction
lint (its rule tests, and no `wa_trap` hit in the ports). It leaves out
the slow prototype fuzz test and the timing budget tests. Before pushing
a change to the prototype, the WebAssembly build or the VS Code
extension, run the slow suites too:

```text
$ scripts/check.sh --full             # adds the prover fuzzer, the budget tests, the wasm smoke test and the VS Code tests
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your
option.
