// The Blind Critic #5 forms: a `for` over `Subset<D>` and `?D`, a
// dependent domain alias, and `(o ??= []).push(x)` on a plain optional.
use std.seq.{compress}

let n: Int = read()
domain Row = 1..=n
domain Col = 0..2
var total = 0
for mask in Subset<Row>:
    var members: List<Int> = []
    for r in mask:
        members.push(r.label)
    let size = sum(1 for r in Row if r in mask)
    let out = [r.label for r in Row if r not in mask]
    print(mask.idx, members, size, out)
    total += 1
print(total)
var cnt: [Subset<Row>]Int = 0
for mask in Subset<Row>:
    cnt[mask] += 1
print(sum(cnt[m] for m in Subset<Row>))
for p in ?Col:
    if let c = p: print("some", c.ord)
    else: print("none")
print([(p ?? Col.first!).ord for p in ?Col])

// `let (domain W, ys) = compress(...)` names the domain `compress` made.
fn tally():
    let (domain W, ys) = compress([5, 5, 2])
    var c: [W]Int = 0
    let w: W = ys.at(5)
    c[w] += 2
    print([c[i] for i in W])
tally()

// `(o ??= []).push(x)` fills a plain optional and goes on with what it holds.
var o: ?List<Int> = none
(o ??= []).push(3)
(o ??= []).push(4)
print(o!)
