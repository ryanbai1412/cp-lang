# 0. Vision and goals

Status: decided. Ryan reviewed `goals.md`, and on 2026-11-19 approved the
additions marked *(added 2026-11-19)* below, in his overnight brief. The
ambition statement is still proposed.

The command is written `cpl` here: that is the binary's working name until
Ryan picks a final one (doc 11.1). Source files end in `.cp`. Other plan
documents still write `cp run`, `cp build` and so on; read those as
`cpl run`, `cpl build`.

## 0.1 The ambition

We are building **the best language there is for competitive programming**,
and the toolchain around it. It should feel so much better than C++ that a
strong competitor switches for good, without giving up any speed, memory or
judge compatibility.

Concretely, at the end of the plan:

- You write a solution that reads like the editorial. Indexing is typed by
  domains, so the classic off-by-one and wrong-array bugs either don't
  compile or fail loudly in a debug run.
- `cpl run d i1` gives you the answer in tens of milliseconds, without
  waiting for g++, and an HTML walkthrough of the whole execution if you ask.
- `cpl stress d` finds a failing input, shrinks it to the smallest one while
  you watch, and names the broken part of your solution.
- Before you submit, the compiler tells you which line will be too slow,
  which table won't fit in memory, and which product can overflow.
- `cpl build` produces one readable C++17 file, under 64 KB, that runs as
  fast as what you would have written by hand, and every judge accepts it.
- The editor shows types, proven ranges, memory use and live sample values
  as you type.
- A docs site and an online playground with real problems teach the
  language in an afternoon.

The core language stays general-purpose; everything contest-specific lives
in the standard library, written with the same powers a user has.

Three things Ryan said about the design as a whole sit above the individual
goals:

- **It should make sense.** "i want every decision to be well justified, and
  i don't want syntax that's just one off special cases. like i like the
  idea of general languages".
- **A small general core, not features bolted on.** "i don't want a language
  where we have like 10 random things bolted on top of each other and each
  of which is implemented by bespoke compiler stuff. i want to ideally make
  most of the user facing behavior also stuff someone can implement in user
  land".
- **Change is cheap now.** "we shouldn't be afraid to massively change
  things". When a port shows friction, or a feature turns out to be a
  special case, redesign it; nothing in this plan is protected by having
  been written down first.

## 0.2 Goals

Goals are tiered. Each has a test we can run.

**P0: the language fails without these.**

- **G1 – Fast by default.** The obvious code compiles to what a strong
  competitor would write in C++: no hidden copies, allocations, virtual
  calls or boxing, and no bounds checks in hot loops when they can be
  proven. Domains, operation values, generators and optionals cost nothing
  at run time. *Test:* every corpus port runs within 1.1x of the original
  C++, and the benchmark suite has no regressions.
- **G2 – Memory-tight by default.** Values take the fewest bytes their
  types allow: widths from ranges, niche optionals, flat grids, 4-byte
  handles. *Test:* every port uses at most the original's memory, and
  `cpl explain` shows bytes per element for every container.
- **G3 – Correct by default.** The easy way is the correct way: typed index
  spaces, overflow caught from ranges, distinct domains, `mut` at call
  sites, exhaustive `match`, optionals instead of sentinels, floor division,
  no uninitialized values or stale per-test state. *Test:* at least 90% of a
  corpus of real past WAs and known pitfalls don't compile, fail a check, or
  can't be written.
- **G4 – Fluent.** You write the algorithm, not the bookkeeping. *Test:*
  ports are shorter than the originals without counting code moved into the
  stdlib, and reviews of the ports find no awkward spots left unfixed.
- **G5 – Great errors and debugging.** Every message is precise and says
  what to do; one root cause gives one error; runtime errors are in source
  terms; debug runs show intermediate values automatically. *Test:* every
  message in a corpus of common mistakes passes the review "would I know
  what to type next?", and `cpl locate` names a seeded bug's part and a
  minimal input in under a minute.
- **G6 – Submittable.** One C++17 file that any judge accepts; under 64 KB
  as a hard limit (warning at 48 KB); deterministic. *Test:* every port
  compiles on GCC and Clang at `-std=c++17` and is under 32 KB.
- **G7 – Coherent and justified.** One obvious way to do each thing;
  features compose without special cases; every decision has a reason
  written down, and no syntax exists for one use only. A feature that needs
  bespoke compiler support has to say why a general mechanism can't
  provide it. *Test:* each new feature is checked against the existing
  ones, the spec's list of special cases stays short, and every row of
  `decisions.md` names its justification. *(Strengthened 2026-11-19.)*
- **G8 – General-purpose core, contests in the library.** *(Promoted from
  P1 on 2026-11-19.)* About 95% of the stdlib, and most user-facing
  behaviour, is ordinary code a user could write with the same performance,
  on top of a small, listed intrinsics layer. The compiler has no
  contest-specific cases: `SegTree`, `@memo`, `@multitest`, typed input and
  the everyday utility functions are library code. *Test:* the intrinsics
  list is short and reviewed; a sample user library (doc 4) reimplements
  stdlib structures at equal speed.
- **G11 – Stable source and pinned local libraries.** *(Promoted from P1 on
  2026-11-19.)* A solution records the compiler version and the exact
  snapshot of every library it used, including a library that is just a
  folder on disk. An incompatible change to that library never changes or
  breaks an old solution, and moving a solution to the new version is one
  command. *Test:* change the sample library incompatibly; every earlier
  solution rebuilds byte-identical, and `cpl upgrade` reports what changed.
- **G12 – Zero-UB compiler trust.** *(Added 2026-11-19.)* The C++ we emit
  has no undefined behaviour, so a judge's optimizer can never turn a
  correct program into a wrong one. *Test:* every test program and port's
  generated C++17 runs clean under ASan and UBSan, and a differential
  fuzzer finds no program whose compiled output differs from the debug
  interpreter's.
- **G13 – An editor that works on broken code.** *(Added 2026-11-19.)* The
  language server, the VS Code extension and syntax highlighting are part
  of the core, not a stretch: "LSP, editor extension, and highlighting and
  stuff should be part of the core overnight bar or else it won't be usable
  lmao". The parser recovers from partial code, so a file with a syntax
  error still gets every other diagnostic, hover and completion. *Test:*
  the LSP fixtures of doc 6.7 pass, including on files truncated at every
  token.
- **G14 – Zero compiler fights under contest pressure.** *(Added
  2026-11-19.)* Standard contest idioms typecheck on the first try. The
  core stays small enough that you never stop mid-contest to satisfy the
  checker, and when a check does fire, the message has the fix. *Test:* a
  list of standard idioms (two pointers, BFS on a grid, DSU, DP over a
  range, coordinate compression and the rest of the cookbook) each compile
  as first written by someone who knows the tour; ports log every time the
  checker had to be worked around, and that log trends to empty.
- **G16 – Low mental overhead.** *(Added 2026-11-19.)* "think about the
  times when code becomes slow to write and people need to take time to
  think about exactly how to write the piece of code. we want to reduce
  this as much as possible by making it just obvious how to write the code
  / making it hard to write wrong code". Wherever C++ makes you stop and
  work out an index, a boundary, an overflow or an initial value, the
  language should make the right code the obvious code. *Test:* each port
  review lists the places where the C++ needed careful thought (off-by-one
  boundaries, sentinel values, index translations, reset-between-tests) and
  records whether the port removed the thinking, kept it, or added new
  thinking of its own; the stdlib's utility functions are driven by that
  list.
- **G10 – A fast toolchain.** *(Promoted from P1 on 2026-11-19: "oh also
  performance is key. also of the toolchain".)* Front end under 500 ms;
  debug runs don't wait for g++; the editor answers as you type; stress
  testing uses every core. Speed of the tools is judged as strictly as
  speed of the generated code. *Test:* latency budgets for check, debug
  run, build, format, LSP responses and stress throughput are measured in
  CI on the corpus ports, and a regression fails the build.
- **G17 – It just works.** *(Added 2026-11-19.)* "things should just work.
  like things should just work. it should be a lovely user experience
  where things just work". Install is one step, a new problem needs no
  setup, commands find the file, the tests and the libraries on their own,
  defaults are right, and nothing ever needs a workaround. This covers the
  language, the tools, the editor and the docs alike. *Test:* a fresh
  machine goes from install to an accepted submission following only the
  first page of the docs; every "I had to work around X" found while
  porting or dogfooding is filed as a bug, not documented as a tip.

**P1: important.**

- **G9 – Predictable.** You can tell what code costs by reading it. Nothing
  secret under the hood: an implicit representation choice is allowed only
  if it is never worse on memory or complexity, and every such choice is
  shown by hover and `cpl explain`.
- **G15 – Clean contest escape hatches.** *(Added 2026-11-19.)* When you
  know better than the checker, there is an explicit, greppable way out
  that costs nothing: `a.raw[i]` to index with a plain `Int`, `unchecked:`
  to drop the remaining index checks in one hot loop, and raw intrinsics
  under the stdlib. They are ordinary, documented features, never hacks.
- **Clean input from types.** *(Promoted from P2 with the input-types
  feature, doc 1.9.)* Typed reads also drive validation, generation and
  shrinking.

**P2: nice to have.** Editor features beyond G13 (debugger panels,
refactorings, live sample values), a REPL (deferred, doc 1.9), and
learnability for C++ competitors reading a solution cold.

## 0.3 Non-goals

- Production or systems programming, and I/O beyond stdin and stdout.
- Running submissions on anything but compiled C++. (The debug interpreter
  is a local tool only.)
- Dynamic typing, reflection, inheritance, pointers.
- Copying C++ semantics where C++ is a known trap (division, overflow,
  uninitialized memory).
- Fewest keystrokes at the cost of clarity.

## 0.4 When goals conflict

**G3/G12 correct > G1/G2 fast and tight > G4/G14/G16/G17 fluent and
frictionless > G7/G8 coherent and general > the rest.** The one exception: a correctness feature
that costs measurable time in hot code must have a zero-cost form, such as
being proven at compile time or checked only in local builds.

G7 and G8 rank below fluency only when they truly conflict. Usually they
don't: when a special case looks like the fluent answer, first look for the
general mechanism that gives the same fluency.

## 0.5 How we'll know it worked

- The user solves real contests in it and doesn't want to go back.
- Every corpus port meets G1, G2, G4 and G6.
- **Line counts, measured two ways** *(added 2026-11-19, Ryan's rule)*. For
  every port we report the % reduction in lines against the original
  submission, and the % reduction against that submission as it would be
  with a perfect C++ contest library behind it. Most of the corpus was
  written with no library at all, so the first number flatters us: it
  mixes what the language gives with what any good library gives. The
  second number is the honest measure of the language. Doc 8.1 says how
  the baseline is counted.
- **Ports are re-solved, not translated** *(added 2026-11-19)*. "you don't
  need to think about just pure translating from c++ to cpl one by one ...
  there's a lot of submissions that could be totally implemented in a
  completely different manner and structure if we do our jobs correctly".
  The corpus of thousands of submissions is the main design tool: mine it
  for the patterns that recur, and judge the language by how those
  problems are best solved in it, not by how the old code maps across.
  "we should repeatedly iterate against it": every design change is
  re-checked against the ports, and the quick Python prototype in `proto/`
  can stay ahead of the Rust compiler when that makes iterating on the
  language faster.
- The bug corpus meets G3.
- Compile, debug-run and stress-test latency meet G10.
