// Source: Codeforces 2138A (Cake Assignment), https://codeforces.com/contest/2138/problem/A
// limits: 2000 ms, 256 MB
// Problem: two piles of 2^k cakes (k <= 60); a move gives half of an even pile to the other.
//   Print the fewest moves to reach (x, 2^(k+1) - x) and the moves (1 or 2, by who gives).
//   Multitest.
// Solution: backwards each move is forced: the giver ends with the smaller pile, so undo it
//   ((x, y) -> (x - y, 2y) if x > y, else (2x, y - x)). Each undo raises the lowest set bit of
//   both piles, so it ends at (2^k, 2^k) within k steps; print the moves reversed. O(k).

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
