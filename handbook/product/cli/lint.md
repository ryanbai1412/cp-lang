# Lint

> 50 minutes in, the solution passes the samples. Before you submit you
> want to know whether anything in it is legal but probably wrong, slow, or
> a C++ habit CP Lang has a better word for.

The editor already shows it: a yellow underline on `var best = -1` with
`L0536: -1 marks "none yet"; ?Int says it in the type`, and a quick fix
that rewrites the declaration and its uses. On the shell, `cpl lint` prints
the same findings, one per root cause, each with its code, the line, why
it matters and the exact replacement. `cpl lint --fix` applies the ones
that are safe unread. Warnings never fail a run; `deny` does. The lint you
disagree with is one line away: `with lint(sentinel_scalar: allow)` around
the code, or `sentinel_scalar = "allow"` under `[lints]` in `cp.toml`, or a
whole group (`idiom = "allow"`). `cpl explain L0536` shows the rule, a
before and after, and how to turn it off.

**Stay quiet when it doesn't fit.** A lint that fires on code that is
already right costs more than it saves: the contestant reads it, judges
it, and learns to ignore the next one. Each lint therefore keeps a
`near_miss.cp`, the closest program it must not fire on, and the corpus
count only goes down. The quiet cases are use cases below, found by walking
each lint over the 877 ports.

**Off the happy path.** The file doesn't type-check: `cpl lint` says so and
reports no clean pass. A library declares its own lint with `@lint`: it
fires, fixes, is configured and explained like a built-in. A bound lint
(`L0302`) names an input that breaks the program: `--save` writes it as a
test case.

**Ideal against today.** The ideal is that every finding is worth reading,
its fix is the one you would write, and no lint fires on a correct idiom.
Today 117 lints have records whose `before.cp`, `after.cp` and `near_miss.cp`
are checked, and fixes never change what a program prints. Gaps: some
lints still read written types instead of the checker's (27 sites), and
idioms keeps finding false positives in ports, each a missing quiet case.

**Against the usual setup.** clang-tidy and Clippy lint syntax and know
little about contest code; neither knows that `-1` in a DP table means
"absent" or that an `adj[u].push(v)` loop is a graph read. CP Lang's lints
read the checker's types and proven ranges, so the suggestion is the
stdlib call that fits.

**Dimensions.** Group (correctness, idiom, style, perf, pedantic), kind (built-in, bound, library), where it shows
(editor, `cpl lint`, `cpl check`, playground), the fix (safe, needs
review, none), level source (default, group, name, `with lint`), file state
(clean, doesn't check, doesn't parse), and the code shape it must stay
quiet on.
