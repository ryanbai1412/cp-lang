# Friction report: noi03p1

## Dream

```cp
let moves = [[(a.at, d)] for a in digits for d in Shape if reshape(a, d) and fixed(a, d)]
    + [[(a.at, x), (b.at, y)] for a in digits for x in loses[a] for b in digits if b != a for y in gains[b] if fixed(a, x, b, y)]
print(if let m = moves.first: eq.with(m) + "#" else: "No")
```

## Changes

The v0.2 port copied the text char by char up to `'#'`, kept the stick pairs as lists
searched with `contains`, accumulated the total while parsing and searched in a `solve()`
with early returns and a `continue`. The rewrite cuts at `'#'` with `split_once`, keeps the
pairs in `Set`s, sums the total from the digits, and lists every fix in one comprehension
(single reshapes first, then pairs), printing the first or `No`.

## Remaining friction

- The parse is still a hand `while` over the characters: no tokenizer for signed numbers.
- Digit values stay `Int`; a `[Shape]` table of stick counts would need a const domain table.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. Corpus tests pass.
