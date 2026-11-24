# Friction report: nccc9s1

## Dream

```cp
@!multitest
let (a, b, c): (0..=1e8, 0..=1e8, 0..=1e8) = read()
print(yes((a + 2 * b + 3 * c) % 2 == 0 and (if c > 0: a >= c else: b % 2 == 0 or a >= 2)))
```

## Already at bar apart from spelling

The v0.2 port had three separate reads and printed `YES`/`NO` by hand. The rewrite reads
one tuple and names the two conditions (`even`, `levels`) before `yes(..)`. 30 random cases
agree with the v0.2 port and 30 with the accepted C++. Corpus tests pass.
