# Friction report: nccc9s3

## Dream

```cp
let s: [P]Char = read()
print(max(Int(s.first! == 'U') + s.adjacent.count(('U', 'U')), s.adjacent.count(('I', 'I'))))
```

## Adjacent pairs from std

The v0.2 port scanned `0..2n-1` with `s[i] == s[i + 1]` and two mutable counters. The rewrite
counts `s.adjacent` pairs equal to `('U', 'U')` and `('I', 'I')`. `count` takes a predicate
rather than a value to compare, so each needs a `p => p == ..` lambda.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. The max case passes.
Corpus tests pass.
