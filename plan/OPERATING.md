# Operating manual

How work on CP Lang runs: what is already decided, and the loop that moves
the project forward. The mission, goals and principles every change follows
are in `AGENTS.md`; read that first. Where each domain stands, its north star and its
next steps are in `plan/DOMAINS.md`.

## Settled decisions

These change only with strong evidence from the corpus.

- Indentation blocks, `and`/`or`/`not`, `elif`, `=>` match arms.
- Rust-style ranges: `0..n` is half-open, `0..=n` inclusive.
- Every integer has a range type. Ranges come from fixed rules (annotations,
  reads, `D.count`, interval arithmetic, hulls), with no flow narrowing.
  Unproven index checks stay in the submission and trap.
- Overflow wraps in release and traps in debug. `/` and `%` floor. `Int`
  excludes `INT64_MIN`, so `?Int` is free.
- `?T` absorbs arithmetic (`none + 5` is `none`); `let ... else`; postfix `?`
  propagation.
- Domains made from runtime data (`domain V = read(1..=n)`, `domain V = n`)
  get one hidden slot per declaration, saved and restored on recursion. A
  value of such a domain cannot leave its block except through an
  existential return type.
- Assume the judge gives a big stack (size from the judge profile). No depth
  inference, no `@depth`, no depth counting. Runtime-sized and large arrays
  live off the stack; returns move.
- `print(yes(b))`, not a bare `print(Bool)`.
- Pointer structures use 4-byte typed handles from a library arena; no pool
  syntax.
- Generics are checked at definition, with excellent errors; libraries can
  attach their own diagnostics.
- Float `==` is rejected by default and configurable.
- Brute, generator and checker are ordinary functions in the same file, not
  special file sections.
- Deferred: program shape (`--- with multitest() ---` is provisional),
  `#set`, testing roles. Keep current behaviour until the rest of the
  language is built.

## The loop

The factory runs continuously, as a pipeline: every stage runs all the time
against the latest `main`.

1. **Port waves.** Port agents take five to ten accepted solutions each from
   the corpus. Each writes the best program in the language, differential-tests
   it against the accepted C++ (interpreter and generated C++), measures tokens,
   runtime, memory and bytes, and files friction reports: the snippet, what
   felt hacky, which goal it hurts, and a proposed general fix.
2. **Triage.** Cluster friction into general mechanisms. Reject one-problem
   helpers. Decide where each fix belongs: core, stdlib, sugar or tooling.
3. **Build.** Lane agents, one per domain and partitioned by crate, implement
   the smallest coherent batch. Every fix lands with the ports that triggered
   it as regression tests, on every tool the feature touches.
4. **Regress.** A scoreboard on every push: ports passing, interpreter vs C++
   agreement, runtime, memory, emitted size, friction count, diagnostic
   goldens. A change that makes a number worse does not land.
5. **Grill.** Cold readers (fresh context, only the goals and the artifact) and
   a two-implementer conformance pair go over whatever changed. Every
   disagreement between implementers is a spec bug.

**Making each domain maximal.** Every domain (spec, core language, codegen
and performance, stdlib, diagnostics, interpreter/debugger/trace, tooling,
judge bundling, editor/LSP, ports and validation, docs and learning) keeps
three things: the maximally ambitious version, a gap list against today, and
a next-step queue with a measurable bar. Each round, ask of every domain:
"what is the maximally ambitious version, and what is the next step?" Every
few rounds a fresh-context session audits the whole language for coherence,
because long-running context drifts.

**The meta loop.** After each round, retro on four numbers: frictions fixed,
ports done, rework rate, and how agents failed. Read agent transcripts for
wasted work. Fix the factory itself: briefs, friction template, scoreboard,
check scripts, reusable skills. Every new agent should start better than the
last. The meta loop may rewrite anything in this file,
the loop and the meta loop included, when the evidence says it should; record
each change and its reason in `research/retros/`.

**Briefing agents.** Give each agent the smallest context that does the job:
the goal, the relevant chapters, the tests to pass, the files it owns. Long
histories poison judgment. Agents report evidence against a commitment; they
never silently override it.

## Working on this repository

- `scripts/check.sh --fast` is the gate before every push to `main`. It
  needs the `wasm32-unknown-unknown` Rust target.
- Interpreter and C++ must agree, including on rejected input. Test both
  whenever semantics change.
- `cpl` is the command; sources end in `.cp`.
- Accepted solutions live in `corpus/` and `subs/`; existing ports in
  `proto/ports/`; each port sits beside its original C++.
