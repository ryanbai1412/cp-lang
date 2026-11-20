Keep the fast solution, a simple trusted alternative, and the input constraints close together. A randomized mismatch is useful only if you can turn it into a small input and understand which part disagreed.

## Memoize the recurrence

`@memo` caches a function by its arguments. Here the domain bounds the state space; the recurrence asks for the previous state only when it exists.

```example memo
```

With input `4`, the states are `1`, `4`, `14`, `45`, so the program prints `45`. The prototype's `i.idx` is a zero-based ordinal helper, not a conversion from an unrelated integer domain.

The cache is not magic algorithm selection. Sparse scalar keys can use a hash-backed cache; finite indexed keys permit dense tables. The prototype also supports `@memo(into: dp)` to name the table and `@memo(order: ascending)` for an explicit ascending evaluation order. Choose an order only when it matches the recurrence's dependencies. Do not read “memoized” as “safe for cycles” or “safe for arbitrarily deep recursion”.

For a real tree DP, read [IOI 2005 Rivers](https://github.com/ryanbai1412/cp-lang/blob/main/proto/ports/ioi05p6.cp). It memoizes depth and the cost of serving a subtree from a selected mill.

## Turn input into a specification

The input **spec** is the type-and-constraint description of valid cases. In the current prototype that description is written as typed `read()` declarations, not a separate `spec` keyword or block.

```cp Input specification · prototype syntax
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]0..=1e9 = read()
```

These declarations say how to read input, how much storage to allocate, and what values a generator is allowed to produce. A `where` condition adds a relation between values. For structured inputs that are expensive to generate by rejection, the prototype accepts a custom `gen:` helper; [Rivers](https://github.com/ryanbai1412/cp-lang/blob/main/proto/ports/ioi05p6.cp) generates a tree directly and checks its drainage invariant.

The approved tooling direction extends this to typed generators and structure-aware shrinkers: a failing tree should shrink by removing leaves, not by deleting arbitrary bytes until it stops being a tree. The full automatic shrinking workflow is still planned, not a feature of the Rust CLI today.

## Keep a brute for one part

The prototype attaches `@brute(fast_name)` to the slow function. This direction matters: name the function being checked, not the brute itself.

```cp CP Lang · prototype syntax fragment
fn fast(a: [V]Int) -> Int:
    sum(a)

@brute(fast)
fn slow(a: [V]Int) -> Int:
    var result = 0
    for v in V:
        result += a[v]
    result
```

This tiny pair assumes the input and domain from the preceding specification. Real brutes are deliberately slower and simpler. A brute for one stage is often more informative than a second implementation of the whole program: it narrows the disagreement to that stage.

## Stress a real port today

Run the prototype against an original C++ solution. The following recipe copies a port out of the tracked directory because prototype build commands stamp the source and write generated files next to it.

```sh
mkdir -p scratch/docs-stress
cp proto/ports/dmopc21c2p1.cp scratch/docs-stress/solution.cp
python3 proto/cp.py stress scratch/docs-stress/solution.cp \
  proto/ports/dmopc21c2p1_orig.cpp 100 8
```

The last two arguments are the number of seeds and the small size bound. The tool compiles the solution, generator, and reference with `g++`, compares tokenized outputs, and prints the seed and input on a mismatch. This is a real local native run, not a simulated example. It does not by itself prove correctness over the full input range.

`python3 proto/cp.py fntest file.cp 20 8` checks the file's `@brute(fast)` and invariant functions on generated calls. `locate` can search small generated inputs and swap a part for its brute to help identify the source of a disagreement. Those prototype tools are separate from the planned polished `cpl stress` command.

## The finished workflow

An input spec describes **legal inputs**. The approved design also calls for co-located `spec` checks that describe **correct behavior**: keep a small trusted implementation beside the fast one, compare intermediate states, and show the first divergence in the trace debugger. That integrated state-diffing experience is planned; today's prototype `@brute` tools are the working starting point, not a claim that the finished `spec` workflow already ships.

The intended `cpl stress d.cp` uses the input spec, checks fast and brute implementations, shrinks a failure while preserving validity, and saves a regression input beside the problem. Tests, brutes, and specification-only machinery must not inflate the submitted C++. The same testing model is intended for standard-library modules and local libraries, not just single solutions.
