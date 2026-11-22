# Agent context

Read this first if you are an agent picking up this project. It holds what
the plan documents don't: who the user is, how the design came about, and
above all the user's own feedback, which is the strongest guide to what
"right" means here.

## The user and the project

Ryan Bai is a strong competitive programmer who writes C++. His accepted submissions are in `corpus/` (3,462 C++ files); `submissions/info.json` has metadata. The project is his personal
one: design and build the best language for competitive programming. It
compiles to C++, and it comes with a full toolchain: CLI, debugger, LSP and
editor plugin, docs site and playground.

The repository is `ryanbai1412/cp-lang` on GitHub (private). Ryan owns it
and asked for work to go **straight to `main`**: no pull requests and no
review gate, so keep `main` green (`cargo test` passes) on every push, and
rebase instead of merging when someone else pushed first.

The command-line tool is called **`cpl`** for now, because `cp` clashes
with Unix `cp`; Ryan will pick the final name. Source files end in `.cp`.
Plan documents written before 2026-11-19 still say `cp run`, `cp build`
and so on: read those as `cpl run`, `cpl build`, and write `cpl` in
anything new.

The `plan/` directory is the source of truth. Older root-level docs
are being folded into it and archived. The `compiler/` course was written for his
education only; it is not part of the handoff, and how to implement things is
up to you. The plan describes the maximally ambitious version of what ships.

## How he wants you to work

- **Work autonomously.** "for the most part tho you kinda have everything you
  need from me to just cook and work autonomously". Make reasonable calls,
  mark them "proposed", and say so. Ask only about real design decisions.
- **Answering his questions comes first, everything else second.** "priority
  1 is answering my questions when they come up but priority 2 is everything
  else".
- **Send documents incrementally** as each one is finished.
- **Prose with example code, not tables.** "not a fan of tables i like
  writing things out with example code and stuff". A table is fine only
  when he asks for one (he did ask for the decisions table).
- **Be concise in chat.** He reads short messages and steers quickly.
- **Do research and experiments instead of guessing.** "for those key
  decisions can you try to do some research and experimentation yourself to
  answer them". Use the corpus and C++ benchmarks (`research/`).
- **Do second passes.** After a batch of documents, re-read them for
  consistency, and look for missing features that would change the design.

## Principles he has stated, in his words

**It should just work, like an Apple product.** From his first message:
"when i use the language it should just feel like the pieces fit together
beautifully and everything just works ... i never should need to feel like i
have to hack stuff together". Translating an idea into code should be "super
easy and natural and quick", and the structure should make it "glaringly
obvious where issues with the code might lie".

**Indexing safety is a P0, but one of many.** "making it so that it's just
super easy for me to write about and reason about things like off by one
errors and weird indexing ... is p0". Later: "when i said index spaces was
P0, that was just an example. we have lots of other P0s".

**Keep the core general-purpose.** "i generally feel like we shouldn't bake
stuff way too competitive programming specific into the language". This is
why per-test scope headers were rejected and multitest became a stdlib
decorator on the file.

**If the stdlib can do it, the stdlib should do it.** "if it can be made into
this, you know, the standard library can support it, we should do it". He
agreed that users should be able to reimplement about 90% of the stdlib
themselves at full speed: "i think we should use this as a good forcing
function".

**No secrets under the hood.** "we should be very careful with things we do
secretly under the hood". He pushed back on a `Set` silently becoming a
bitset and a `Map` silently becoming a slot array: "i kinda feel like it's
cool but also feels like you just almost always want to be explicit
anyways". Dense forms are explicit types.

**Memory matters.** "memory is pretty sensitive. like the diff between 4
bytes and 8 bytes is sometimes huge esp in data structures for caching
reasons". Also: "we have a lot of stuff that ends up hurting perf. e.g.
subsets can we do this in a way where we use compact like u32 or u64".

**Submissions under 64 KB are critical.** "btw under 64kb is critical lmao".
Later (2026-11-21) he tightened the goal: "always possible to get 1.1x with
more manual tuning, and all solutions under 32kb".
He also noticed when a port came out *more* bytes than the C++.

**Rewrites must rethink, not translate.** "it's not just about translating
line to line. it's about thinking about the logic and thinking about how
you'd actually express what you're trying to do in this new world". He
doesn't use enums in C++ "cuz c++ enums suck", so ports should use ADTs,
optionals and domains where they fit.

**Line counts must be honest.** "moving stuff to stdlib code doesn't really
count as lines of code reductions lmao".

**Errors and debugging are a key goal.** "one key goal is really good error
and debugging messages ... and ofc everything should feel very nice".

**Every decision is justified; no one-off special cases.** "i want it to
feel like it makes sense. i want every decision to be well justified, and i
don't want syntax that's just one off special cases. like i like the idea
of general languages". Each row of `decisions.md` should be able to say
why, and a construct with a single use is a sign the general mechanism is
missing.

**A small core, with user-facing behaviour buildable in userland.** "i
don't want a language where we have like 10 random things bolted on top of
each other and each of which is implemented by bespoke compiler stuff. i
want to ideally make most of the user facing behavior also stuff someone
can implement in user land". Earlier, in the same spirit: "we should make
sure the language is still tastefully and thoughtfully designed and
decisions make sense and we try to avoid things that are purely built into
the core and are like special cases".

**Don't be afraid to change things massively.** "we shouldn't be afraid to
massively change things". Nothing in the plan is protected by being
written down. When porting shows a cleaner single abstraction, refactor
the design, update `decisions.md` with the reason, and move on.

**Reduce mental overhead.** "think about the times when code becomes slow
to write and people need to take time to think about exactly how to write
the piece of code. we want to reduce this as much as possible by making it
just obvious how to write the code / making it hard to write wrong code".
Any hesitation while porting is a signal: simplify the language, or add a
stdlib helper.

**The corpus is the main tool, and ports are not translations.** "one
powerful tool we have is the repository of the thousands of submissions i
have. you don't need to think about just pure translating from c++ to cpl
one by one btw. probably there's a lot of submissions that could be totally
implemented in a completely different manner and structure if we do our
jobs correctly". And: "we should repeatedly iterate against it. and if
needed we can also e.g. start out with a quick prototype python compiler or
smth for fast iteration speeds and fine tuning the language to make it very
good". `proto/cpc/` was that prototype; it was deleted once `cpl` overtook
it (`research/MIGRATION.md`).

**Performance is key, including the toolchain's.** "oh also performance
is key. also of the toolchain". Generated code must match hand-written C++
(G1, G2), and the compiler, the debug runs, the LSP and stress testing must
feel instant (G10, now P0).

**Things should just work.** "and things should just work. like things
should just work. it should be a lovely user experience where things just
work". A workaround anywhere, in the language, the tools, the editor or the
docs, is a bug to fix, not a tip to document (G17).

**Measure line counts two ways.** "you should both measure % of lines of
code reduction and also % of lines of code reduction assuming i had a
perfect c++ cp library (which i don't cuz most of these were written
without any library at all)". Doc 8.1 has the counting rules.

**The standard library is broad, and full of small helpers.** "i think we
also want to implement a lot of util functions. just all sorts of functions
that would make my life and mental load in all the submissions i wrote way
easier". It must also be easy to test the standard library and generate its
docs, and to do the same for your own library.

## Decisions he made directly

Syntax: all of `archive/style-decisions.md` (1-33, included in the handoff; plan/17-syntax.md is the current form), including indentation blocks
with `{}` only for inline expressions, `=>` match arms ("let's use =>"),
`and`/`or`/`not`, `@memo` decorators, `[V] v => expr` tables, every string
literal interpolates, no semicolons, `Int` capitalized, floor division ("we
should just get rid of the negative remainder bug"), and `@get` for getters
("it feels like it should be an explicit @ annotation").

Semantics:

- **Ownership:** variables own and parameters borrow, with `mut` both at the
  declaration and at the call site ("yea mut at both fn param time and fn
  call site seems great"), and no `mut` on method receivers. He likes the
  model's simplicity and was worried something might be missing; the research
  says structs that keep borrows are the gap (about 2% of files).
- **Copies:** big values need an explicit `.copy()` ("altho ofc they can
  still move").
- **Records:** anonymous records for one-off shapes ("yea i really want
  anonymous records").
- **Grids:** indexed `g[r, c]`.
- **Monoids:** an operation over a type, not a type: "a monoid is an
  operation over smth", and "this is broadly true of all segtree stuff".
- **Purity:** no `pure fn`. "what's the point of knowing whether or not a
  function is pure?" Instead, tools record I/O as inputs and outputs, and
  `@memo` rejects fns that do I/O.
- **Program structure:** a file is a function body, `@multitest` goes on
  the file, and `static let`, top-level `return`, and program vs library mode
  follow from that. "we should assume that a ton of problems are multitested
  so we should make the multitest and single test experience the same ...
  but yea it is annoying to have to stuff every single thing inside the same
  function."
- **`Int.MIN`:** `-(2^63 - 1)`, so `?Int` has a niche.
- **Maps:** `Map` is ordered; `HashMap` is the opt-in fast one.
- **Cost estimates** for TLE, MLE and overflow are warnings only.
- **Domain checks** have explicit, greppable escape hatches.

Tooling:

- **Compiles** under 500 ms ideally; "running in debug mode doesn't even
  have to go through gcc".
- **Scratch inputs:** support `cp run d i1 i2`; "i usually just keep my input
  files in i{1..5}", per problem.
- **Libraries:** updating a library a file uses must be easy and intuitive,
  including libraries that are just local paths.
- **Version stamps** in build outputs, so old code can be rebuilt ("not a
  huge priority would just be nice to have").
- **Traces:** debug runs trace each stage. "this --trace feature is going
  to be key. we just need ways to be able to like, filter it down", "make
  sure it's not too noisy", and the output should be "a super good well
  designed html that walks you through the entire execution of the program
  ... with the program embedded next to it". For now, design the language so
  everything the renderer needs is captured; the UI comes later.
- **Snapshots:** save the input of a specific stage and rerun just that stage.
- **Stress testing** "should be actually smart and intelligently shrink while
  keeping you updated with the latest smallest test case".
- **Editor support:** "we definitely need a top notch quality LSP and editor
  extensions for vscode at least".
- **Testing parts:** problems with several moving parts must be easy to
  write and test part by part.
- **Standard library:** "we should build a good default standard library
  (doesn't need to get super fancy but should have the basics)".
- **Validation:** "super important to have good validation as well and mostly
  ensure performance".

## More of his inputs (from the full message history)

- **Priorities:** clean input reading is nice but "probably a p2". Things
  like binary search, neighbour helpers, prefix sums and modint are stdlib,
  not core; the core must make them easy to write. "inf is good and nice
  builtins are good".
- **Staged solutions:** make it easy to express the usual flow "first do A,
  then take that output and do B, then ... run D with C and C reversed", and
  to test each part on its own. Also "very clean ways to represent things
  like out of bounds checking and edge cases and returning failures".
- **Brute force and stress:** "making it super easy to swap in and out brute
  force and non brute force implementations and super easy to stress test
  specific implementations with great tooling as well is huge". The input
  spec idea "would be huge". Running solve on symmetries is good.
- **Checkers and interactors:** "we should probably have a checker as well",
  and "a really clean way to do interactors would be great".
- **State:** "defer, scoped undo, and even being able to abstract over
  rollbacks somehow would be sick"; the typed pool "could be great";
  generators wanted; bound checks must not crush perf.
- **Laws:** algebraic laws should come "naturally as part of the type
  system".
- **`#ifdef LOCAL`:** an equivalent should be easy (minor).
- **Early returns:** he asked about IIFEs and `return` inside `do:` blocks.

## What he wants shipped (his overnight brief)

- A Rust compiler with the full language and standard library, a working
  type checker, and a very large test suite so correctness is certain.
- 100-500 of his trickiest submissions ported, with bundling verified, no
  performance loss measured objectively, and zero cost checked by reasoning
  too ("is Option costing us extra bytes"). Ports must be "truly nice to
  code"; "any awkwardness and friction is a sign that we should change the
  design". Iterate the design on his submissions, tastefully.
- Crystal-clear error messages and a pleasant experience throughout.
- Tooling usable for real contests today, respecting output limits, with a
  local library on disk next to the stdlib that sometimes gets
  backwards-incompatible changes; formatter, linter, stress; a debug mode
  that records everything into a beautifully designed HTML trace.
- Beautiful documentation, learning resources, and a nice way to learn the
  language.

## Calls from the overnight handoff, answered 2026-11-19

- **CLI name:** `cpl` for now, `.cp` files. He will pick the final name
  (doc 11.1).
- **Local library:** build a fresh sample library. "you should build your
  own local library. my cp-library is kinda meh but you can use some
  inspiration from other popular ones". Use it to test library versioning
  and breaking changes.
- **Editor scope:** the LSP, the VS Code extension and syntax highlighting
  are part of the core bar, not a stretch: "LSP, editor extension, and
  highlighting and stuff should be part of the core overnight bar or else
  it won't be usable lmao".
- **Trace UI:** the old `mock/trace-mock.html` is rejected and is not in
  this repository. "the trace mock is absolutely terrible we should NOT
  model it off of it and instead design our own experience with our own
  beautiful design system". Design the trace viewer fresh; doc 5 still
  holds for what the trace must contain.
- **Goals:** the additions to doc 0 marked 2026-11-19 (zero-UB trust, zero
  compiler fights, the editor, pinned local libraries and a fast toolchain
  as P0, clean escape hatches, low mental overhead, "it just works").
- **Proposals:** the defaults in `decisions.md` §8: typed input generators
  and shrinkers, contracts and `const` are in and the REPL is deferred; all
  four research recommendations are adopted; the leans on strings, modint,
  explicit type arguments, labeled `break` and the orphan rule are taken.
- **How to work from here:** "i think really you should just work
  autonomously and there'll always be more stuff to do and so you should
  just manage yourself and continue working on more stuff".

## Concerns he raised that the design must keep answering

- **The borrow rules must never be confusing** (2026-11-19). "one worry i
  have is that with our borrow checker semantics the language ends up being
  very difficult and unintuitive to understand and there's the 1% of cases
  where you just don't understand what's going on". His method: repeatedly
  have fresh agents hunt for the simplest programs where the rules surprise
  (a rejection that reads as wrong, or an error that doesn't explain
  itself), fix the rule or the message, and keep every such program in a
  comprehensive test suite. A case nobody can explain in one sentence is a
  design bug (G14, G16), not a documentation gap.
- `@memo` might not give enough control (table layout, clearing, order).
- `fn foo(a: List<Int>)` borrows while `let a: List<Int>` owns; is that
  confusing? (Answer so far: the call site makes it clear, and `mut` marks
  every mutation.)
- Whether acyclic imports work for people in practice.
- Packaging: one config per machine may be wrong; you may want one project
  per judge, and libraries from elsewhere on the machine, with old solutions
  still building after a library changes.

## State at the time of writing

- **Prototype:** a Python compiler in `proto/cpc/`, since deleted. It was partial: no
  generators and no rollback, it still parses `pure fn`, and it still runs
  multitest through `fn solve()`.
- **Research:** `research/NOTES.md` has the data and
  `research/RECOMMENDATIONS.md` has proposals for borrows, generators, domain
  strictness and checks.
- **Approved on 2026-11-20:** `plan/19-index-and-lends.md` (1.10): one
  mechanism for `domain`, `Str`, `Grid` and `Perm`, projections (`lends`),
  `law disjoint` and lexical exclusivity; also Euclidean `/` and `%`.
  Outcomes are in `decisions.md` §9. The prototype implemented the surface
  rules (`proto/cpc/lends.py`, `crates/cpl-cli/tests/cases/lend/`, the `lend_*`, `proj_*`
  and `index_*` files in `crates/cpl-cli/tests/cases/bad/`); 19 §11 lists what it leaves
  out. The Rust compiler does not have any of it yet. A red-team pass
  added the cases of 19 §5.6 (11 to 23) as `crates/cpl-cli/tests/cases/redteam/` and
  `crates/cpl-cli/tests/cases/bad/rt_*`: calls that write captured variables, loops
  lending what they walk, left-to-right evaluation, and the ends of `Int`.
- **Reviewed on 2026-11-19:** the four proposals in
  `plan/18-key-features.md` (folded into the plan as 1.9) and the research
  proposals above; outcomes are in `decisions.md` §8.
- **Rust workspace:** `Cargo.toml` at the root and `crates/` hold the
  skeleton of the real compiler and tools; `crates/README.md` says which
  crate owns what. They are empty shells with a passing test each.
- **Next steps:** the Rust compiler, stdlib, tools, LSP and editor
  extension, trace viewer, docs, and corpus ports, built in parallel on
  `main`.

## His inputs on 2026-11-21 (the 0.3.0 spec and the hill-climb)

- The plan: "write down this entire spec", "maybe do one review of it with an independent
  reviewer", "read through like 100 solutions, write the dream, compare it with the spec and
  hill climb the spec from here (you should be the owner of this)", "then write down the
  spec-compliant solutions to those 100 things", "build out across all domains, including
  compiler but also lsp, etc."
- The two files: "`dream.cp` is the ideal version, written from the C++ and the editorial idea
  without looking up the spec or running the checker. `spec.cp` is the spec-compliant solution,
  written after the comparison and checked with `cpl check`."
- Organization: "we should stay organized. very organized with the spec and also organized with
  our corpus (e.g. maybe metadata json files and just structured output and stuff). you own the
  codebase so feel free to clean things up."
- Ownership: "you own the language. feel free to make large changes and decisions as needed if
  you feel it's for the better"; "you just own hill climbing the spec yourself honestly".
- Costs: "i'd prefer we generally adopt a guideline that you need parentheses for anything
  that's not O(1)" ("g.transpose should be .transpose()").
- Indexing: "by default all indexing uses the same domain"; "library functions should always
  return results in S.keys or whatever".
- Overlapping mutation (`a[l] += inc(mut a[l])`, `xs[0] = xs.pop()!`): "things we should lint
  for as opposed to flat out ban".
- Release builds: "we should default to not trapping in release ... when you run on the judge,
  you kinda only care about perf".
- Layout: the LSP should make it easy "to see for the user what the size of things being stored
  actually turns out to be".
- Naming: asked to think about variant casing "independently and not be bounded by path
  dependence"; chose PascalCase (`Some`, `None`; decision 157).
- On the blind dreams: liked a domain-keyed `R.group` ("not this specific version but smth like
  it"; decision 158), asked for `(V, V).all((u, v) => ...)` (decision 159), and agreed that
  interval pieces are "a nice concept to have" (decision 162).
- Asked whether the spec is "solid enough that another agent reading it would be able to
  basically end up with the same language"; the honest answer is in `spec/STATUS.md`.
- "please add everything nonsensitive / additional info to the repo and also make sure it's easy
  for agents to onboard onto."
