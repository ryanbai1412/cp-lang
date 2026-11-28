// expect: ok, output "s 519\ns 761\n-62\nw7 s 522\ns 555\ns 192\n70\n11\ns 231\nw2 s 975\ns 881\nw2 w8 w8 s 218\ns 168\ns 79\ns 498\n-12\ns 618\n97\ns 793\nw5 w7 w3 6\ns 961\nw4 s 150\n-71\nw4 s 553\ns 426\n1\n-6\ns 230\nw0 s 578"
// input: "1 40 0\n"
// model: flush
// compare: stdout
// engines: next
// `flush()` against a brute force that never flushes, over seeded random output: `Int`s,
// `Str` words and `write`s with no newline, a flush after a random subset of them, and none at
// the end. Reads `seed n side`: side 0 flushes, side 1 doesn't; the two outputs agree (R10.4.1).
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
let side = read<Int>()
var rng = Rng(seed0)
for _ in 0..n:
    let k = rng.int(0, 2)
    if k == 0: print(rng.int(-99, 99))
    elif k == 1: write("w{rng.int(0, 9)} ")
    else: print("s", rng.int(0, 999))
    let f = rng.chance(0.5)
    if side == 0 and f: flush()
