# Decisions

**Status:** 2026-11-24. One line each. "Ryan" means he decided it;
"working" means the design assumes it and he may overrule; "open"
means evidence is still missing.

## Language and library

| # | Decision | Status | Evidence |
|---|---|---|---|
| 1 | Stored function values stay. A value is one of a closed set of closures; a call is a branch. No `std::function`, no function pointer. | Ryan | closed sets beat `std::function` on time and bytes in every prototype |
| 2 | A function value that would contain itself is a compile error. | Ryan | slice t04, t05, t24 |
| 3 | A struct method whose struct holds 2 or more closures in a field is specialised per closure, with the branch on entry. | working | 1.19x–1.44x without, 1.03x–1.09x with |
| 4 | `min`, `max`, `abs`, `pop_count` are declarations. Maps, sets, sorting, lists and arrays are written in CP Lang. | Ryan | slice: `List` and `Array` over four storage operations |
| 5 | Containers beat the C++ built-ins or are within 1.1x. | Ryan | ordered hash map passes except lookup (1.06x–1.15x) |
| 6 | `HashMap` and `SortedMap`; no plain `Map`. `HashMap` iterates in insertion order. | Ryan (names), working (order) | the ordered layout was the fastest tested |
| 7 | Sorting is stable. The spec promises the result and "each key computed once, in order", not the comparisons. | Ryan (stable), working (rest) | one merge sort fails 12 of 14 workloads; a family passes 12 |
| 8 | Floats match bit for bit across engines. `pow` may take 1.3x. | Ryan, working (`pow`) | audit 3 |
| 9 | A NaN stops a checked run; release follows IEEE with one stated result. | working | engines disagree today |
| 10 | Layout is chosen in the source, never in secret. | Ryan | |
| 11 | A library function with a C++ body also has a CP Lang body. | Ryan | |
| 12 | Only the library that declares a trait or a type may implement that trait for that type. | Ryan | |
| 13 | A std addition that collides with a user's function is an error with a fix. | Ryan | |
| 14 | Editions exist; low priority. | Ryan | |
| 15 | The set of decorators is closed; a library cannot declare one. | working | the spec says closed; the design had said open |
| 16 | A generator that calls itself is allowed and costs one indirect call per item. | working | slice t19 |
| 17 | Two lends in one call end last-lent first. | working | the only order a recursive lender can give; today's compiler does the opposite |
| 18 | Pushing a function into a list of functions received from a caller is an error for now. | open | slice t16; the alternative shares one set across callers |
| 44 | A literal key into a view is E0305 only when the checker knows the view's keys (today a `s[lit..lit]` cut); any other view checks the key as it runs (R0101). | working | R07.4.5; core-07/take-drop-clamped, core-04/slice-drop-keeps-source-keys-traps |

## Engines and tools

| # | Decision | Status | Evidence |
|---|---|---|---|
| 19 | One fast local engine: an interpreter everywhere (8x of C++), plus machine code on desktop (5x). The playground is about 12x. | Ryan (fast engine), working (bars) | 5x–7x interpreter, 2.4x–4.9x machine code in prototypes |
| 20 | A small reference interpreter runs the program before any rewrite, for tests only. | working | slice: it caught every divergence so far |
| 21 | Native streaming traces; stepping backwards on the engine only. | Ryan | slice: the same events from the interpreter and from C++ |
| 22 | Every engine bounds-checks storage always, also in `unchecked:`. | working | under 5% in the engine prototype |
| 23 | The playground's size cap is reset once from a measurement. | Ryan | |
| 24 | Language changes keep landing during the migration. | working | about 400 language commits a day |

## Architecture

| # | Decision | Status | Evidence |
|---|---|---|---|
| 25 | Forms: syntax, declarations, typed tree, generic IR, instance IR in three stages. One facade for tools. | working | four audit rounds kept it; the slice has every form but the facade |
| 26 | A link step between generic IR and instances answers the whole-program questions: instances, function-value sets, hidden run-time values, and whether a lent value is changed. | working | audit 4; slice milestones 2 and 4 |
| 27 | The typed tree is a tree of its own. | working | slice |
| 28 | In IR, anything that can fail or has an effect is its own statement. | working | slice |
| 29 | Event points are placed after the link step, on request. | working | slice milestone 5 |
| 30 | Inside a solution file, declarations and statements are computed on demand over each other. | working, not built | audit 4 finding 2 |
| 31 | Whole-program steps choose representations and do not reject, with two closed exceptions. User errors are per unit, from summaries. | working, not built | round 5 contracts review F5 |
| 32 | Owned-or-lent is a mode on every heap-holding slot and part of the concrete type. | working, not built | round 5 F1 |
| 33 | A closure holds a local `var` by reference and is then lent; returning it is an error. | working | spec R05.10.1; slice t26 to t29 |
| 34 | Run-time values of a type are a witness: passed down, captured, and returned. Reverses spec R02.9 / R13.2. | working, partly built (domain sizes down only) | round 5 F4; `mod_rt.cp` |
| 35 | Function-value sets leave the type and become a link-time representation. | direction, not built | round 5 F3 |
| 36 | The oracle is the spec's conformance suite. Old against new is a triage signal with three recorded verdicts. | working | round 5 migration review F1: old and new agree on 8 of 24 |
| 37 | The new pipeline refuses anything not on its supported list, by exhaustive match. "Accepted" means equal to the expected result. | working, not built | round 5 migration F2: 136 of 332 accepted conformance programs wrong |
| 38 | The facade is built first, over today's compiler. | proposed, waits for Ryan with the order | round 5 migration F5 |
| 39 | An IR operand is a constant, a copy or a borrow. Ownership picks borrow or copy and records it; an unwritten copy is an error or a cost note. | working, not built | round 5 slice review F1: `xs[i + 1]` quadratic |
| 40 | A run-time value of a type is keyed by its binding within an instance, not by its declaration. | working, not built | round 5 slice review F3: `44` for `51` |
| 41 | Agreement between paths that share lowering is not evidence of correctness. Tests need expected output, a cost bound and sanitizers. | working | round 5 slice review F7 |
| 42 | Factoring instances that differ only by closure is a required rewrite. | working, not built | round 5 slice review F4: 80 KB at 50 lambdas |
| 43 | A set of functions that would contain itself: compile error (Ryan) or a boxed representation (two reviews). | waits for Ryan | round 5 contracts F3, slice review F4 |
