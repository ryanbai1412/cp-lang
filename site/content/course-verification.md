A fast solution is a claim. This chapter shows how to check that claim against something slower and obviously correct, for recurrences, for problems with many valid answers, and for interactive problems, and how a failing case is shrunk until the bug is easy to see.

> **Status.** Two different things are described here, and the labels keep them apart. **Working in the prototype today:** `@memo` and `@memo(into: table)`, `@brute(f)` slow references for functions, `@invariant(rev)` and `@invariant(shuffle)`, random inputs generated from typed `read()` declarations, and the `proto/cp.py` commands `fntest`, `stress`, and `locate`. **Approved but not implemented:** `@checker`, `protocol` and `@interactor`, structure-aware shrinking, the integrated `cpl stress` command, co-located `spec` state diffing, and first-divergence views in the trace debugger. The prototype has no `spec` keyword; its input specification is the typed `read()` calls. The [testing reference](testing.html) and [plan/15-program-structure.md](https://github.com/ryanbai1412/cp-lang/blob/main/plan/15-program-structure.md) have the details.

## Write the recurrence, not the cache

A dynamic-programming solution is a recurrence plus a table. `@memo` lets you write only the recurrence; each argument is computed once, and the table is indexed by the argument's domain, so there is no hash map and no sentinel `-1`.

```example memo
```

Read it as math: `f(0) = 1`, and `f(i) = 3 * f(i - 1) + i` when `i` has a predecessor. `i.prev` is `none` at the first index, which is the base case, so "forgot the base case" and "read `dp[-1]`" are not expressible. Note that `domain I = 0..n` is half-open: with input `n` the last index is `n - 1`.

```exercise verify-memo-output
```

`@memo` is a cache, not a cycle solver. If `f(i)` calls `f(i)` again through some chain, the program has a bug in its recurrence; the cache does not invent a fixed point. When you need the table itself afterwards (to reconstruct a path, say), name it with `@memo(into: dp)` and read `dp[i]`, which is `?Int`: `none` for states never visited.

## Keep a slow reference beside the fast part

The most effective test in CP is a brute force that is too slow to submit but too simple to be wrong. In CP Lang it lives in the same file, attached to the function it checks:

```cp CP Lang · prototype-tested
@brute(f)
fn f_slow(i: I) -> Int:
    var x = 1
    for k in 1..=i.idx:
        x = x * 3 + k
    x
```

`@brute(f)` says "`f_slow` must agree with `f` on every argument". The prototype's `fntest` generates program inputs from the `read()` types, calls both functions on random arguments, and stops at the first disagreement. With a planted off-by-one (`x * 3 + k - 1`) it reports:

```text
$ python3 proto/cp.py fntest recurrence.cp
seed 2:
MISMATCH f vs @brute f_slow
args: i=1
f: 4
f_slow: 3
```

Two choices make brutes useful rather than ceremonial:

- **Brute a part, not the whole program.** A brute for one function tells you *which* function is wrong. A brute for the entire solution only tells you the final answer differs.
- **Make the brute different in kind.** A loop instead of a recurrence, a scan instead of a data structure, all subsets instead of a greedy. If the brute shares the fast version's idea, it shares its bug.

Brutes, invariants, and laws are stripped from the submission; they never cost judge time or code size.

```exercise verify-brute-part
```

## Typed reads are the input specification

Random testing needs valid random inputs. In CP Lang the input format and its constraints are the same declarations that read the input:

```cp Input specification · prototype syntax
let n: 1..=20 = read()
domain P = 1..=n
let a: [P]-100..=100 = read()
```

In generation mode the same lines *produce* a random `n` in `1..=20` and `n` values in `-100..=100` instead of reading them, so the format, the constraints, and the generator cannot drift apart. `where` adds a constraint across values, and generated cases that violate it are rejected and counted, which is the "rejected by where" number in the prototype's output.

The prototype's `stress` command runs this today against a reference C++ solution, starting from small sizes:

```sh
mkdir -p scratch/docs-stress
cp proto/ports/dmopc21c2p1.cp scratch/docs-stress/solution.cp
python3 proto/cp.py stress scratch/docs-stress/solution.cp \
    proto/ports/dmopc21c2p1_orig.cpp 100 8
```

Copy the source first: the prototype stamps a version comment into the file it builds. On a mismatch it prints the seed, the input, and both outputs.

## Shrink to the smallest failing case

A random failure with `n = 37` and twelve queries is evidence, but not yet an explanation. What you want is the smallest input that still fails.

**Today,** `proto/cp.py locate solution.cp reference.cpp` tries sizes from 1 upward, so the first failing input it finds is already small. Then it swaps each `@brute`'d function for its brute, one at a time, and reports the first swap that makes the output correct: that function is where the bug lives.

**Planned,** `cpl stress` shrinks structurally, because it knows the `read()` types. It removes array elements and renumbers the domain, deletes queries and test cases, and lowers values toward the bottom of their ranges, while keeping every constraint (`l <= r`, distinct values, a valid tree) true at every step. A candidate only counts if it fails *the same way*, at the same first differing stage or the same crash line, so shrinking never wanders off to a different bug.

```exercise verify-shrink
```

## Many valid answers: a checker, not a diff

"Print any valid coloring" breaks output comparison: your answer and the brute's can both be right and still differ. The approved design adds a **checker**, a function whose parameter type is the output format:

```cp CP Lang · design fragment
@checker(output: ?[V]Color)        // "-1", or one color per vertex
fn valid(c: [V]Color) -> Bool = all(c[e.u] != c[e.v] for e in es)
```

The tools parse your output with the same typed reading used for input, and the checker can see the input `let`s such as `es`. The brute still matters: it settles **existence**. If the brute says there is no answer (`none`, printed as `-1`), your output must be `-1` too; otherwise your output must parse as `[V]Color` and pass `valid`. So the checker only answers "is this a correct answer", never "does an answer exist", which keeps it short and hard to get wrong.

Until checkers land, test multi-answer problems in the prototype by writing a brute for a *canonical* answer, such as the lexicographically smallest, or by comparing a derived quantity (the number of colors used, the total cost) instead of the raw output.

```exercise verify-checker
```

## Interactive problems: protocol and query budget

An interactive problem is a conversation with a hidden judge. The approved design describes the conversation once, as a `protocol`, and the judge's side as an ordinary struct:

```cp CP Lang · design fragment
protocol Guess:
    @wire("?") fn ask(x: 1..=1e9) -> Ordering
    @wire("!") @final fn answer(x: 1..=1e9)
    limit ask: 30

@interactor(Guess)
@derive(Gen)                         // the hidden value is generated
struct Judge(secret: 1..=1e9):
    fn ask(mut self, x: 1..=1e9) -> Ordering = x <=> self.secret
    fn answer(mut self, x: 1..=1e9) -> Bool = x == self.secret

let judge = Guess.connect()
judge.answer(first_true(1..=1e9, x => judge.ask(x) != Less)!)
```

- **The wire format is declared, not hand-written.** On the judge, `judge.ask(x)` prints `? x`, flushes, and reads the reply. Forgetting to flush is not possible.
- **The budget is part of the protocol.** `limit ask: 30` is checked locally on every run: a 31st `ask` fails the case and prints the call log. Binary search over `1..=1e9` needs at most 30 questions, since 2^30 is just over 10^9, so the solution above fits exactly.
- **Locally, it is one process.** `judge` is a `Judge` value, so running and stress testing play the whole interaction without pipes. `@interactor(Guess, adaptive)` lets the judge pick answers that keep as many secrets possible as it can, which finds worst cases a random secret misses.

None of this runs in the prototype yet. Today, test interactive logic by putting the strategy in a function that takes the judge's answers as a parameter, and `@brute` that function against a direct computation.

```exercise verify-query-budget
```

## Find the first divergence

When a fast and a slow version disagree on the final answer, the useful question is *where they first disagree*. The approved debugger records every stage of a debug run and puts fast and brute side by side:

```text Planned cpl stress output · not implemented
  FAIL at case 313, shrunk to:
    n = 2  a = [7, 7]  q = 1  qs = [(l: 2, r: 2)]
  stage          fast            brute
  last_seen      [none, 1]       (same fn)
  answer         [0]             [1]          <- first difference
```

Everything before the first difference is known good, so the bug is in the stage that produced it, and its inputs are the rows above. The HTML trace view jumps straight to that row. The prototype's `locate` gives the coarse version today: it names the first `@brute`'d function whose replacement fixes the smallest failing input.

## Common mistakes

- **A brute that copies the fast idea.** Write the brute from the problem statement, not from your solution.
- **Testing only large random cases.** Small sizes find most bugs and are readable. Both the prototype and the planned tools start from size 1 for this reason.
- **Comparing raw output on a multi-answer problem.** Use a checker (planned) or compare a canonical form.
- **Counting queries by hand.** Put the limit in the protocol and let every local run enforce it.
- **Debugging the final answer.** Look for the first stage that differs; the bug is there, not at the end.
