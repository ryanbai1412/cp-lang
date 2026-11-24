# Friction report: mccc4j3

## Dream

```cp
let score: [V]1..=100 = read()
let before = score.prefix()
print(V.find(i => 2 * before[..=i] == score.sum()) ?? "Andrew is sad.")
```

## Prefix folds need a start

`before.fold(V.first!..=i)` spells the prefix up to `i` with an explicit first position. An
open `..=i` range on `Prefix` would drop `V.first!`.

## One print for two types

The answer is a position or a message, so it is an `if let ... else` over two `print`s. A
`??` that unifies `?V` with `Str` for printing is out of scope for the type system, so it stays as it is.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`, half the cases
balanced). The max case (n around 1e6, balanced) runs in 0.01 s. Corpus tests pass.
