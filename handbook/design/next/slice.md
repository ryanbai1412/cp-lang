# The thin slice

**Status:** 2026-11-25. Lives in `crates/cpl-next`. The commit gate runs it on
every conformance case; `crates/cpl-next/tests/passing.txt` and `wrong.tsv`
are the current counts.

The slice is real code through every layer after the parser, for a
small part of the language. Its job is to find out where the contracts
are wrong before anything is migrated. It reuses the repo's parser
(`cpl-syntax`) and builds the rest.

## What it is

```
parse (cpl-syntax) → declarations → typed tree → generic IR → link step → instance IR (raw)
                                                                              │
                                                    required rewrites → canonical IR
                                                                              │
                                reference interpreter (runs raw and canonical) ┴ C++17 emitter
```

`run_tests.sh` runs each program on the interpreter before the
rewrites, on the interpreter after them, and as compiled C++, and
compares output, exit code and the error that fired.

## What it covers

| Part of the language | Test | Shows |
|---|---|---|
| `List<T>` and `Array<D, T>` written in the language over four storage operations | all | no compiler knowledge of containers |
| traits, impls, a generic fn with a bound | t01 | impl chosen per instance |
| a function in a field of a generic struct, reassigned | t02 | fails to build for the judge today |
| `List<Int -> Int>`, mutually recursive through the list | t03 | sets inside type arguments |
| a closure that captures a function of its own set | t04 | rejected at link |
| a recursive fn that wraps its function parameter | t05 | rejected at link |
| overflow, bad index, division by zero | t06–t08 | same error on all three |
| a fn that returns a function value | t09 | the caller's set is "what it returns" |
| DSU; map, fold, compose | t10, t11 | ordinary code |
| a domain with a run-time size, in a generic fn | t13 | hidden values |
| a new domain per recursive call, kept by closures | t14 | a global slot would be wrong |
| a domain declared in a top-level loop | t15 | each round has its own |
| generators with `break`, `continue`, `return`, nesting | t17 | regions |
| a lender whose tail repairs a total | t18 | regions |
| a generator that calls itself | t19 | needs an indirect call |
| pushing to a list while it is walked or passed, also through a function value | t12, t20–t22 | rejected at link; accepted today through a function value |
| a stored operation wrapped by `dual`, branches joined | t23, t24 | works with a new name; reassigning the same variable is an error |

Not in it yet: moves, drops and last use, a lend that spans
statements, trace events, per-statement rechecking, optional rewrites,
the bytecode engine, most of the grammar.

## What it changed in the contracts

| # | Finding | Contract |
|---|---|---|
| 1 | The typed tree works as a tree of its own. Lowering needed nothing else. | 3 |
| 2 | Function-value sets work as hidden set parameters solved by the checker, with two conversions: a closure enters a set, and a set enters a larger one. | 3, 5 |
| 3 | A top-level function variable gains members from later statements, so the file is one inference scope for sets. | 3 |
| 4 | Adding a function to a list of functions that came from a parameter or a call has no cheap conversion, because containers cannot be widened in place. Open: error, or share one set across all callers. | 3, 5 |
| 5 | Every operation that can fail or has an effect is its own statement. C++ leaves argument order open, so nested checked arithmetic could fail in another order. | 4 |
| 6 | A binding inside a top-level block is captured by value. Only file-level bindings are shared. | 4 |
| 7 | A by-value parameter is printed as `const&` so that passing a list costs nothing. That is correct only if the callee cannot write the same list. | 4, 7 |
| 8 | A domain's size is one pure node in generic IR. The link step finds which instances need which sizes, adds hidden leading parameters, and closures capture them. Instance identity does not include them. | 5 |
| 9 | Arrays store their length, so hidden sizes are needed only by code that names the domain, never to copy a value. | 5 |
| 10 | The region form runs in both backends. A generator stops when the body leaves; a lender finishes first. | 4, 7 |
| 11 | A generator that calls itself takes the caller's code as two words and calls it indirectly. | 7 |
| 12 | Each new IR node meant edits in seven tree walks. The IR needs one shared traversal. | 4 |
| 13 | Ownership's "is this value changed while lent" is answered in the link step, from what each whole instance reads and writes. A call through a function value is the union over its set, which is exact there. | 4, 5 |
| 14 | A third conversion: a struct whose function fields grow is rebuilt field by field. The branches of an `if` are joined into a new set, not merged. | 3 |

## Real corpus programs (2026-11-24)

The slice accepts 30 of the 1,535 corpus programs, unchanged (24 at first: 22 from `corpus/newcomer/` and 2 ports). `compare_old.sh` runs each on three inputs four ways (today's compiler, the slice's interpreter before and after the rewrites, the slice's C++): 90 runs, all agree. On the first input 22 programs finish and 2 stop with the same error on every path.

What was added to get there: range types on scalars (an `Int` checked once where the value is made), arrays indexed by an unnamed range (`[0..n]T`, `[n]T`: each gets a domain of its own), `read()` by the expected type and `read<T>()`, an `Int` index into an array (checked), `for v in a` over an array, printing lists and several values, a tuple pattern over `read()`, and `@!multitest`.

Findings:

15. **A decorator the compiler does not implement must refuse the program.** The slice skipped `@memo`, and the program ran correctly but in exponential time. Decorators change meaning or cost, so none is ever skipped.
16. **A body ending in `return` needs a "never" type.** Without it lowering added a second return of the wrong type; the interpreter never reached it and the C++ failed to compile. Only the compiled path caught it.
17. **An unnamed range in a type is a domain declaration.** `let a: [0..n]Int` declares a domain as a side effect of a type, so "type conversion is pure" (contract 2) is false in a body. The slice names such ranges before converting the type.
18. **`Int` indexing into `[D]T` is the common case in real programs,** not the typed-index case the slice was built around. Each is a run-time check until the range facts exist, so the facts layer decides whether real programs meet the speed bar.

The refusal list, most common first, is in `census.sh`'s output. The top reasons now: types not in the slice (tuples, optionals, `Str`, floats), statements (`match`, `while let`, compound patterns), and domains that do not start at 0.

## Round 5 (2026-11-25)

19. **A closure must share a local `var`, not copy it** (spec R05.10.1). The slice copied, and a four-line counter printed `0` where today's compiler prints `2`. All three slice paths agreed with each other, because they share the lowering. Agreement between paths cannot catch an error made before they split; only the spec's expected output can. Fixed: a reference capture, and an error when such a closure is returned (t26 to t29).
20. **The slice accepts what it does not support.** A reviewer ran it on the conformance suite: 332 accepted, 136 wrong, for example `xs.len()` runs where the spec says E0304. Refusal has to be by a closed list. Not fixed yet.
21. **The slice is pinned to an old checkout.** Two parser commits after it was built break it (`CmpOp::Order` removed; `@intrinsic` now outside the closed decorator set, which the slice's `std.cp` is built on).

Known gaps in the reference capture: a heap value is still copied, storing a lent closure through a `mut` parameter is not an error, and the borrow check does not see a write made through a reference.

## Round 5, third report: 143 adversarial programs

Report: `round5/slice-report.md`. It reviewed the 24-test slice, before
finding 19 was fixed.

22. **Hoisting a failing operation copied every earlier by-value argument.** `xs[i + 1]` deep-copied `xs`. The IR needs a borrow operand, chosen by ownership (contract 4). No test timed anything, so nothing saw it. Not fixed.
23. **The ownership check is unsound in three ways:** a generator reading a global while the loop body writes it (interpreters `6 6`, C++ `103 6`), `two(mut a, mut a)`, and a lender on a global whose body replaces the global. Not fixed.
24. **Hidden sizes were keyed by declaration,** so a generic function called with its caller's own local domain mixed two sizes (`44` for `51`). Not fixed.
25. **Closed sets reject ordinary programs:** a list of a monoid and its dual, `m = m.dual()` in a loop, a function pushed into a list from a parameter, 300 lambdas. Second review to say so. Waits for Ryan's decision on a boxed fallback.
26. **Checker holes that crash a backend:** `yield` in a closure, `print` of a function, a lender with `lend` in a loop. Not fixed.
27. **Three paths that share lowering agree by construction.** The raw and canonical interpreters never differed on 143 programs, so the second adds little. Findings 19, 22 and 24 were wrong on all paths.
28. **No timing and no sanitizers in `run_tests.sh`.** Where no copy is hit, the slice's C++ is 1.2x to 1.8x of hand-written on four benchmarks; the lender segment tree does not finish.
