// CF 1358D The Best Vacation: the best trip ends on the last day of some month,
// so for each month take the shortest wrapping run of months ending there with
// at least x days and drop the surplus from the front of its first month.
let n: 1..=2e5 = read()
let x: 1..=2e11 = read()
domain M = 1..=n
let d = read<[M]1..=1e6>()

let tri = (k: Int) => k * (k + 1) / 2
let days = d.scan(+)
let hugs = ([M] m => tri(d[m])).scan(+)

fn around(pre, w: RingSlice<M, Int>) -> Int:
    let (l, r) = (w.start.l, w.end.r)
    if w.start <= w.end: pre[r] - pre[l]
    else: pre[M.last!.r] - pre[l] + pre[r]

let shortest = d.ring.minimal(Window(+), ok: w => (w.all ?? 0) >= x)
var best = 0
for w in shortest.present():
    let surplus = around(days, w) - x
    best max= around(hugs, w) - tri(surplus)
print(best)
