# Friction report: joi16fp2

## Dream

```cp
let express: [E]1..=stations = read()
let gains = [served(from, at) for (from, to) in express.adjacent() for at in stops(from, to).take(spare + 1)]
print(sum(first of each stretch) - 1 + last + gains.top(spare).sum())
```

## `pairs()` on a table is every pair, not adjacent ones

The first rewrite used `for (from, to) in express.pairs()` to walk consecutive express stops
and was wrong: on a table `pairs()` gives every `i < j` pair. The port goes back to
`for e in E: let Some(f) = E.at(e + 1) else: continue`. A named `adjacent()` (or
`windows(2)`) on tables and lists would make the consecutive walk one spelling, and a lint on
`pairs()` over a sorted domain table might catch the mix-up.

## Taking the largest k

`gains.sorted().rev.take(spare).sum()` sorts every gain to keep the top `spare`. A
`top(k)` / `nth_element` spelling would show the partial-sort cost.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The max case
matches the C++. Corpus tests pass.
