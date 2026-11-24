# Friction report: tle18p5

There is no dream.cp. The port is the ideal for the stdin form of the problem: std's `a.lis()` gives the positions of one longest strictly increasing subsequence.

## Dream

```cp
@grader mod pure:
    pub fn solve(n: Int, get_item: (Int) -> Int):
        let best = [n] i => get_item(i) |> lis
        ...
```

## The judge's function interface becomes stdin
- **Where:** line 10: "The judge header's N and get_item become stdin; print_num becomes print". `orig.cpp` gets the same rewrite.
- **What felt wrong:** the problem is an interface task (no mutation, items read through `get_item`). The verified program is a stdin program, not the one the judge would link.
- **Goal it hurts:** predictable path onto the judge.
- **Ideal code:** a `@grader` module whose entry point receives `n` and `get_item`.
- **General fix:** grader modules with function parameters, the same gap as research/frictions/ioi25_obstacles.md. Owner: core/spec.
- **Severity:** low.

## Measurements

- **Tokens:** port 49, accepted C++ 423 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 2 MB, accepted C++ 0.00 s / 3 MB, with the same output.
- **Emitted C++:** 12216 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
