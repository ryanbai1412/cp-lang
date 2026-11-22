// NOIP 2001 Splitting a Number (proto/ports/noip01p2.cp), with a planted bug: a part of size 1 is
// never dropped once total equals parts, so n = k counts zero ways.
let n: 1..=200 = read()
let k: 1..=6 = read()

fn ways(total: Int, parts: Int) -> Int:
    if parts == 0: return Int(total == 0)
    if total <= parts: return 0
    ways(total - 1, parts - 1) + ways(total - parts, parts)

@alt(ways)
fn ways_slow(total: Int, parts: Int) -> Int:
    fn count(left: Int, parts: Int, least: Int) -> Int:
        if parts == 0: return Int(left == 0)
        sum(count(left - p, parts - 1, p) for p in least..=left)
    count(total, parts, 1)

print(ways(n, k))
