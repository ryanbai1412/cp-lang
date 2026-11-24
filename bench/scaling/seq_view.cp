// n: 4096
let n: Int = read()
let ys = [i for i in 0..n]
var s = 0
for r in 0..262144:
    let k = r % n
    let sl = ys.take(n - 1)
    s += sl.len // timed: seq.Slice.len
    s += Int(sl.is_empty) // timed: seq.Slice.is_empty
    s += sl.keys.len // timed: seq.Slice.keys
    let rv = ys.rev
    s += rv.nth(k) // timed: seq.Rev.nth
    s += rv.take(5).len // timed: seq.Rev.take
    s += rv.drop(5).len // timed: seq.Rev.drop
    s += rv.take_last(5).len // timed: seq.Rev.take_last
    s += rv.drop_last(5).len // timed: seq.Rev.drop_last
    s += rv.src.len // timed: seq.Rev.src
    let st = ys.step(3)
    s += st.len // timed: seq.Stepped.len
    s += st.nth(k / 3) // timed: seq.Stepped.nth
    s += st.at(k / 3 * 3) // timed: seq.Stepped.at
    let sk = st.keys // timed: seq.Stepped.keys
    s += sk.len // timed: seq.StepKeys.len
    s += Int(st.is_empty) // timed: seq.Stepped.is_empty
    s += Int(sk.contains(k)) // timed: seq.StepKeys.contains
for r in 0..1024:
    s += ys.rev.scan(+).all // timed: seq.Rev.scan
for r in 0..1024:
    s += ys.take(n - 1).items.count(p => p.1 >= 0) // timed: seq.Slice.items
    s += ys.step(1).each().count(x => x >= 0) // timed: seq.Stepped.each
print(s)
