# Friction report: noi01p1

## Dream

```cp
print(count(not known.unite(b, a, Kind(claim - 1)) for (claim, a: ?Animal, b: ?Animal) in read(k)))
```

## Hand weighted union-find becomes `PotentialDsu<Animal, Mod<3>>`

The v0.2 port kept its own `up`/`ahead` arrays, recursive path compression and `% 3` fixes.
The rewrite types the kinds as `Mod<3>` and lets std `PotentialDsu.unite` answer whether a
statement is consistent: it returns false exactly when the statement contradicts what is
known. A statement naming an animal past n fails the `Animal.at` let-else. `x` and `y` stay
`Int` because the statement gives no bound and out-of-range labels are part of the input.

## Verification

`gen.py` is new: a hidden kind per animal, about a third of the statements lying (wrong
relation, a label past n, x eats x), and a max case with n = 5e4, k = 1e5. 30 random cases agree with
the v0.2 port and 30 with the accepted C++. Corpus tests pass.
