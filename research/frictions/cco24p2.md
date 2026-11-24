# cco24p2 frictions

## Dream

```cp
elif a.is_permutation(Flavour):
```

## Domain as a permutation target

`a.is_permutation(Flavour)` fails inside `std.seq` ("a `for` can't walk over a `Flavour`"),
so the port collects the domain first: `a.is_permutation(Flavour.collect())`. A domain is a
sequence everywhere else; the std generic should accept it directly.
