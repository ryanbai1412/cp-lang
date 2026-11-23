# Stress

> 70 minutes in, WA on test 9 and the samples pass. You suspect the greedy
> skips a case, but you can't see which. You want an input where it is
> wrong, small enough to check by hand.

You already have the slow, obviously right version: a few lines that try
every option. You put it in the same file, `@alt(program) fn brute()`, and
run `cpl stress d.cp`. Within a second it prints the input it broke on, cut
down to `n = 2  a = [3, 3]`, what the brute printed, what you printed, and
the first wrong token. With `--explain` it adds the line that made the
first wrong value: `look at line 14: lo = mid leaves lo unchanged`. The case
is saved as `d.tests/stress-4.in` with the brute's answer beside it. You
fix the line; `cpl test d` reruns the saved case green, and `cpl stress d`
again says `1000 cases agree`.

**Off the happy path.** You have no brute yet, only a WA: stress still
runs, and a case fails on a runtime error or the time limit, shrunk and
saved the same way. Many answers are right: a `@check fn` judges the output
instead of a brute. The problem is interactive: the file's `@interactor`
plays the judge. The input shape is one stress can't make, a string over
its own alphabet with a length tied to `n`: it says which `read()` and why,
and a `@gen fn` makes the cases instead. The brute itself crashes: stress
stops and blames the brute, not you. The solution hangs: the case fails as
a time-out, and `--explain` names the loop whose state stopped changing.

**Ideal against today.** The ideal is one command from WA to the wrong
line, and one from the fix back to a clean run. Today stress, shrinking,
saving, checkers, interactors, the reader-checked generator and `--explain`
work as above. The gaps, each a use case: `--explain` is a flag rather than
part of every failure; after a fix, `cpl stress` doesn't rerun the saved
failures before new ones; `@brute(f)` and `@alt(f)` are two spellings of one
idea; 30 input shapes in the corpus can't be generated yet
(`crates/cpl-stress/tests/generate-gaps.txt`); and nothing measures that
generated cases reach the edges (`n = 1`, all equal, the maximum value).

**Against the usual setup.** A C++ contestant's stress loop is three files
and a shell script: `gen.cpp`, `brute.cpp`, `while diff`. It needs a
generator written per problem, finds a failing input but never shrinks it,
and has no idea which line is wrong. Python's Hypothesis and Haskell's
QuickCheck shrink, but need the input's shape written again as a strategy.
`cpl stress` takes the shape from the `read()`s the solution already has,
so the first stress run of a problem costs only the brute.

**Dimensions.** What judges a case (brute program, brute part, checker,
interactor, nothing), where inputs come from (the `read()` types, a `@gen
fn`, an outside generator, saved cases), the program's shape (one test,
multitest, interactive), how a case fails (WA, runtime error, time limit,
the brute failing, input the reader rejects), what the contestant reads
(text, JSON for an editor, a trace, an explanation) and when (first run,
after a fix, in the editor, in the browser).
