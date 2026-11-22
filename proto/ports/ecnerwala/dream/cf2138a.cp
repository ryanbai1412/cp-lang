// Codeforces 2138 A — Cake Assignment
//
// Starting from `(2^k, 2^k)`, forward operation 1 moves half of `x` to `y`
// (`(x, y) -> (x / 2, y + x / 2)`) and operation 2 moves half of `y` to `x`.
// Running backwards from target `(x, 2^(k + 1) - x)` (`1 <= x < 2^(k + 1)`),
// the strictly smaller pile must be the one that was just halved:
// - If `x > y`, undo op 2: `(x, y) -> (x - y, 2 * y)`.
// - If `x < y`, undo op 1: `(x, y) -> (2 * x, y - x)`.
// The sum `x + y = 2^(k + 1)` is invariant and `ctz(x) == ctz(y)` strictly
// increases by 1 on every step (since `x, y > 0` and `x != y`), so the walk
// terminates at `x == y == 2^k` in `<= k` steps; printing `ops.rev` restores
// the forward sequence.

@!multitest
let k: 1..=60 = read()
var x: 1..=2 ** 61 - 1 = read()
var y = (1 << (k + 1)) - x
var ops: List<Int> = []
while x != y:
    if x > y:
        ops.push(2)
        x -= y
        y *= 2
    else:
        ops.push(1)
        y -= x
        x *= 2
print(ops.len)
if not ops.is_empty: print(ops.rev)
