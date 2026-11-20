// Corpus submission 4811954: IOI 2009 Salesman (day-grouped DP + two range-max SegTrees)
let n: 1..=5e5 = read()
let up: 1..=1000 = read()
let dn: 1..=1000 = read()
let home: 1..=500001 = read()

domain Fair = 1..=n
domain Pos = 0..=500001

let fairs: [Fair](t: Int, l: Int, c: Int) = read()

type FairItem = (l: Int, c: Int)
var by_day: Map<Int, List<FairItem>> = Map<Int, List<FairItem>>()
for f in Fair:
    let x = fairs[f]
    if by_day.get(x.t) == none:
        by_day[x.t] = []
    by_day[x.t].push((x.l, x.c))

let last_day = 500001
if by_day.get(last_day) == none:
    by_day[last_day] = []
by_day[last_day].push((home, 0))

fn opt_max(a: ?Int, b: ?Int) -> ?Int:
    if let x = a:
        if let y = b:
            return max(x, y)
        return x
    b

var lst = SegTree<Pos, ?Int>(none, (a, b) => opt_max(a, b))
var rst = SegTree<Pos, ?Int>(none, (a, b) => opt_max(a, b))

lst.set(Pos.at(home), home * dn)
rst.set(Pos.at(home), -home * up)

for (day, raw_cur) in by_day:
    var cur = raw_cur.copy()
    cur.sort()
    let m = cur.len
    domain I = 0..m

    var mval: [I]?Int = none
    for i in I:
        let loc = cur[i.idx].l
        let p = Pos.at(loc)
        var best: ?Int = none
        if let lv = lst.query(Pos.first, p):
            best max= lv - loc * dn
        if let rv = rst.query(p, Pos.last):
            best max= rv + loc * up
        mval[i] = best

    var psum: [I.bounds]Int = 0
    for i in I:
        psum[i.r] = psum[i.l] + cur[i.idx].c

    var opt: [I]?Int = none

    // Sweep 1: enter from left-most or intermediate fair, visit suffix, finish heading left
    var val_l: [I]?Int = none
    var copt: ?Int = none
    for i in I.rev:
        let loc = cur[i.idx].l
        copt max= -(up + dn) * loc + psum[i.r]
        if let mv = mval[i]:
            val_l[i] = copt! + (up + dn) * loc - psum[i.l] + mv
    var copt2: ?Int = none
    for i in I.rev:
        let loc = cur[i.idx].l
        if let vl = val_l[i]:
            copt2 max= -up * loc + psum[i.l] + vl
        if let c2 = copt2:
            opt[i] max= c2 + up * loc - psum[i.l]

    // Sweep 2: mirror pass finishing heading right
    var val_r: [I]?Int = none
    var dopt: ?Int = none
    for i in I:
        let loc = cur[i.idx].l
        dopt max= (up + dn) * loc - psum[i.l]
        if let mv = mval[i]:
            val_r[i] = dopt! - (up + dn) * loc + psum[i.r] + mv
    var dopt2: ?Int = none
    for i in I:
        let loc = cur[i.idx].l
        if let vr = val_r[i]:
            dopt2 max= dn * loc - psum[i.r] + vr
        if let d2 = dopt2:
            opt[i] max= d2 - dn * loc + psum[i.r]

    for i in I:
        if let oi = opt[i]:
            let loc = cur[i.idx].l
            let p = Pos.at(loc)
            lst.set(p, oi + loc * dn)
            rst.set(p, oi - loc * up)

let ans = lst.query(Pos.at(home), Pos.at(home))! - home * dn
print(ans)
