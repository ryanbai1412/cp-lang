# Friction report: nccc8s1

## Dream

```cp
let near = s.edits(alphabet, deletes: 1, replaces: 1, inserts: 1)
for w in near.sorted().distinct() if w != s: print(w)
```

## Three edit kinds, three comprehensions

The v0.2 port pushed every edit into one mutable list from a loop with an `if i < n` guard
inside. The rewrite builds deletions, replacements and insertions as three comprehensions,
each over its own range, so the guard goes away.

## `s[0..i] + c + s[i..n]` re-slices both halves

Each edit rebuilds the whole string. That is fine for |s| <= 100. A `Str.insert(i, c)` /
`replace(i, c)` / `remove(i)` returning a new string would say the edit by name.

## No accepted C++

The only AC submission is Java, so `orig.cpp` is a hand-written C++ reference (a set of
edits). 30 random cases agree with the v0.2 port and 30 with `orig.cpp`. Corpus tests pass.
