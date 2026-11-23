# 9. Contexts, checks and build modes

Status: draft for 0.3.0. Sources: archived `design/spec/17-contexts.md`,
`18-checks-lowering.md`; decisions 100, 145, 148. Decision 148
supersedes the older rule that unproven checks stay in submissions.

## 9.1 Contexts

A context is a library-declared record of compile-time values with
defaults. It replaces a flag per feature with one mechanism:

<!-- spec-only: the prelude's `io` context declaration -->
```
context io:
    base: 0 | 1 = 0
    sep: Str = " "
    answers: (Str, Str) = profile.answers
    precision: ?Int = profile.precision
```

Any block sets one:

<!-- spec-only: fragment over `A` with a `...` body -->
```
with io(base: 1)                  // bare line at the top of a block: the rest of it
with checks(bounds: off):         // block form
    for x in A: ...
read(base: 0)                     // a call argument beats every context
```

- Precedence: call argument, then the innermost `with`, then outer ones,
  then the default. A bare `with` line must come first in its block
  (the formatter enforces it). (R09.1.1)
- Values are compile-time constants. A runtime value is a parameter or a
  fresh type (`type Z = Mod(p)`), never a context: a table built under
  one modulus must not be read under another. (R09.1.2)
- Contexts are **lexical**: a body sees the contexts where it is written.
  A function that should follow its caller declares `uses ctx`; the
  caller's value is then an implicit compile-time argument, passed on to
  further `uses` callees. `uses` is inferred only for nested fns and
  lambdas; top-level fns (public or not), trait methods and modules
  declare it, and one that calls a `uses` fn without declaring it passes
  its own lexical value. (R09.1.3)
- Cost: one specialized copy of each `uses` function per distinct value;
  `cpl explain` lists them and the bundle-size warning counts them. (R09.1.4)
- Standard contexts: `io` (chapter 10), `checks` (below), `lint` (one
  field per lint: `with lint(float_eq: allow)`), `stress` (default sizes,
  time budget). User libraries declare their own. (R09.1.5)

## 9.2 Build modes

| Mode | Used by | Checks |
|---|---|---|
| checked | `cpl run`, `cpl stress`, `cpl test`, the interpreter, `cpl build --checked`, C++ with `-DLOCAL` | every unproven check traps (R09.2.1) |
| release | `cpl build`, `cpl submit`, the bundle on the judge | no runtime checks unless opted in (R09.2.2) |

**The rule for every check** (decision 148): on the judge only valid
programs matter, so release optimizes for them. Valid programs behave
identically in both modes and in the interpreter, and the lowering
never adds undefined behaviour to a valid program. A program that would
trap in a checked build is invalid, and in release it gets whatever its
C++ does: usually a wrong answer, possibly a crash or C++ undefined
behaviour (an unchecked index past the end reads out of bounds), the
same as the hand-written C++ it replaces. Where avoiding UB is free the
lowering still does it (wrapping arithmetic through unsigned casts,
masked shifts, no uninitialized reads), so `-O2` cannot miscompile the
valid code around a bug; it never adds a branch or a clamp to do so.
To keep a check on the judge, opt in (§9.4).

**Checks never call user code.** A checked build makes exactly the calls
to user functions, closures and input that release makes, in the same
order: a check may only inspect values the program already computed. So
an interactive query, a counter in a closure or a `@cache` sees the same
history in both modes. (R09.2.3)

## 9.3 What is checked

Every checked operation answers three questions: what it means, what the
compiler can prove, and what each build does.

| Check | Proved away when | Checked build | Release |
|---|---|---|---|
| index `a[i]` with `i: D` on `[D]T` | always (the type is the proof) | none | none (R09.3.1) |
| `D(x)` from an `Int`, raw index, `List` index | range facts show it fits | trap with value and bounds | unchecked (R09.3.2) |
| store into a range type | the value's range fits (`x % 7` into `0..7`) | trap naming value and range | truncates to the width (R09.3.3) |
| `Int` / `I128` overflow | interval arithmetic proves it fits | trap with operands | wraps (R09.3.4) |
| `x!` on `None` | flow proves `Some` | trap naming the place | reads the niche as a value (R09.3.5) |
| division by zero, negative shift, `sqrt` of a negative | the divisor's range excludes 0 | trap | defined fallback (0) (R09.3.6) |
| `assert c` | n/a | trap with the message | skipped (R09.3.7) |
| `@cache` cold-mode check | n/a | stress compares cold and cached runs | none (R09.3.8) |
| `law` promises | n/a | stress samples them | none (R09.3.9) |

Each row has one check name, used by `with checks(...)`,
`release_checks:` and diagnostics: `bounds` (the first two rows),
`ranges`, `overflow`, `unwrap`, `div`, `asserts`, and `reads` below.
`@cache` and laws are stress-only and have no name. This is the one list;
other chapters link here.

Reads (`read<1..=n>()`) are input validation: they are checked in
checked builds and, like other checks, skipped in release unless
`release_checks: [reads]` is set.

## 9.4 Turning checks on and off

<!-- spec-only: tour with `...` bodies -->
```
with checks(ranges: on, overflow: on, bounds: on):   // also in release
    ...
with checks(overflow: off):                          // also in checked builds
    ...
unchecked:                                           // every check off, every build
    ...
```

- `with checks(...: on)` keeps those checks in release for a block or
  file. A judge profile can keep them for a whole submission with
  `release_checks: [ranges, overflow]`. (R09.4.1)
- `with checks(...: off)` removes a check in checked builds too, for a
  hot loop whose stress runs are too slow. (R09.4.2)
- `unchecked:` is the greppable escape: a statement block with every
  check off in every build. (R09.4.3)
- Lints are contexts: `with lint(name: allow)` silences one in a scope,
  and is the only in-code spelling: `@allow(name)` is removed syntax (E0503,
  decision 243). Lints are declared by the library that owns the names they
  mention; the linter has no hard-coded stdlib names. (R09.4.4)

## 9.5 Trust tiers

Every fact the compiler relies on is in one tier (`05-ownership.md`):

1. **proved**: the type checker, range analysis or borrow rules
   establish it; no runtime cost. (R09.5.1)
2. **trapped**: not proved; checked builds trap, release assumes it. (R09.5.2)
3. **promised**: the author states it and tooling samples it: `law`
   declarations, `@cache` correctness. Codegen never optimizes on a
   promise alone beyond the documented specialization it selects; a
   false promise gives a wrong answer, never undefined behaviour. (R09.5.3)
4. **unsafe**: `@trusted:` blocks and `@cpp` inline C++. These are the
   only places a contestant can write undefined behaviour, and
   `cpl explain` lists every site. (R09.5.4)

## 9.6 Recursion and the stack

The bundle runs `main` on a thread whose stack comes from the judge
profile (default 1 GB). There is no depth inference or counting; a
stack overflow crashes as in C++, and the checked runtime prints the
call trace. Runtime-sized arrays and static arrays over 4 KB never go on
the stack.
