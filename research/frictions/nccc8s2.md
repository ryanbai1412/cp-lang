# Friction report: nccc8s2

## Dream

```cp
let s: [domain P]'a'..='z' = read()
print(prod(Z(s.count(c) + 1) for c in 'a'..='z'))
```

## Letter tally as a domain table

The v0.2 port used `List.filled(26, 0)` indexed by `s[i] - 'a'`, plus stale prototype notes.
The rewrite tallies into `[Letter]Int` and takes the product with `prod`.

## `Char` read, then `Letter(c)`

The string is read as `Char` and converted per letter. Reading the characters as a
`'a'..='z'` range would put the bound in the type.

## No accepted C++

The only AC submission is Java, so `orig.cpp` is a hand-written C++ reference. 30 random
cases agree with the v0.2 port and 30 with `orig.cpp`. Corpus tests pass.
