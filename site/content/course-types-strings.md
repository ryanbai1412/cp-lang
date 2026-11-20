In C++, competitive programmers juggle `long long`, `double`, `pair<int, pair<int, int>>`, `std::string`, and tagged unions written by hand with parallel arrays. CP Lang gives you four numeric types, records and enums, exhaustive `match`, and an ASCII-only `Str` type with O(1) indexing. String slicing currently copies the selected bytes.

## Numbers, Euclidean division, and floating-point guardrails

CP Lang has four numeric types: `Int` (signed 64-bit), `U64` (wrapping unsigned 64-bit, handy for rolling hashes), `I128` (signed 128-bit for overflow-free intermediate products), and `F64` (64-bit IEEE double). Scientific literals without a decimal point like `1e9` and `1e18` are `Int`; write a decimal point (`0.0`, `1e-9`) for `F64`.

Integer division `/` and modulo `%` are **Euclidean** everywhere: `a % b` is always in `0..|b|`, even when `a` is negative. You never need C++'s `((a % m) + m) % m` idiom. When the compiler proves `a >= 0` and `b > 0`, it emits raw C++ `/` and `%` with zero overhead.

```cp CP Lang 0.2.2
let rem = (-1) % 5          // 4 (not -1)
let q = (-1) / 5            // -1 (satisfies q * 5 + rem == -1)

let x: F64 = F64(7) / 2.0   // explicit F64(...) for Int variables; literals unify
let k: Int = floor(x)       // Int(x) on F64 is a compile error: pick floor/ceil/round/trunc
print("{x:.6f}")            // bare print(x) on F64 is an error; never prints -0.00
```

Three `F64` rules prevent classic contest Wrong Answers:

- `NaN` and `Inf` trap immediately at runtime in local runs, which makes `<` a strict total order on `F64`.
- Bare `print(x)` on an `F64` is rejected at compile time — write an explicit precision like `print("{x:.9f}")`, and CP Lang normalizes `-0.0` to `0.0` automatically.
- Compare floats with `abs(a - b) < eps`; `==` and `!=` on `F64` compare exactly (exact `== 0.0` is fine).

```exercise types-euclidean
```

```exercise types-f64-rules
```

## Records, structs, and payload enums

Use a **record** (`type`) for lightweight named tuples and a **struct** when you want methods. Use an **enum** for closed sets of states or tagged queries — every variant can optionally carry typed data.

```cp CP Lang 0.2.2
type Edge = (u: Int, v: Int, w: Int)

struct Box:
    lo: Int
    hi: Int
    fn len(self) -> Int: self.hi - self.lo

enum Query:
    Update(pos: Int, val: Int)
    Ask(l: Int, r: Int)
```

`match` is exhaustive — if you add a new variant to `enum Query`, the compiler immediately points at every `match` that needs to handle it. You can also bind fields and write guards inline.

```example enum_match
```

```exercise types-enum-match
```

## Control flow: `elif`, parallel assignment, and labeled breaks

CP Lang keeps control flow minimal with one spelling per construct:

- Always write `elif`, never `else if`.
- Inline conditionals use the same syntax as block `if`: `let sign = if x >= 0: 1 else: -1`.
- Swap or update multiple variables in one step with parallel tuple assignment — the entire right-hand side is evaluated before writing to the left: `(a, b) = (b, a % b)`.
- Break out of nested loops cleanly with a loop label (`'outer: for r in Row: ... break 'outer`) instead of adding boolean `found` flags.
- Filter loops directly with `for v in V if ok(v):`. For a list or string, iterate `i in 0..a.len` and access `a[i]` when you need both the position and value.

```cp CP Lang 0.2.2
var (a, b) = (48, 18)
while b != 0:
    (a, b) = (b, a % b)

let n = 3
let grid: [0..n, 0..n]Int = 7
let target = 7
var found: ?(Int, Int) = none
'search: for i in 0..n:
    for j in (i + 1)..n:
        if grid[i, j] == target:
            found = (i, j)
            break 'search
print(a, found)
```

## ASCII strings (`Str`) and tagged states

Contest inputs are ASCII. In CP Lang, `Char` is a single byte (`'a'`), `Str` is a value-semantic `0..s.len` byte array with zero UTF-8 overhead, and both indexing (`s[i]`) and slicing (`s[l..r]`) run in `O(1)`.

```cp CP Lang 0.2.2
let s: Str = read()
if s.len >= 3:
    let first_digit = s[0] - '0'
    let prefix = s[..3]
    print(first_digit, prefix == "abc")
```

When a problem has two strings and you want the type checker to stop you from indexing string `a` with a position from string `b` (for example in LCS or edit-distance DP), opt into a nominal character domain with `let word: [domain S]Char = read()`.

```example ascii
```

```exercise types-ascii-count
```

```exercise types-str-vs-domain
```

For character-encoded grids, read ASCII characters and map them to a state enum explicitly. Automatic `@tags` input is specified but is not yet supported by the execution backend:

```cp CP Lang 0.2.2
let h: 1..=50 = read()
let w: 1..=50 = read()
domain Row = 1..=h
domain Col = 1..=w
enum Tile:
    Empty
    Wall
    Start
    End

let grid: [Row, Col]Char = read()
for r in Row:
    for c in Col:
        let ch = grid[r, c]
        assert ch in ".#SE"
        let tile = match ch:
            '.' => Empty
            '#' => Wall
            'S' => Start
            _ => End
        if tile == Start: print(r, c)
```

## Write a program

```exercise types-write-digits
```
