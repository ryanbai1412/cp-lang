// Runnable port (proto/cp.py lib-run) of dmopc21c9p2.cp
// original: corpus/submissions/4563973.cpp

let t: Int = read()
for _ in 0..t:
    let a: Str = read()
    let b: Str = read()
    // `pending` is a queue whose live part is pending[head..].
    var pending: List<Char> = []
    var head = 0
    var matched = 0
    for c in a:
        pending.push(c)
        while pending.len - head >= 2 and pending[pending.len - 1] == pending[pending.len - 2] and pending.last != 'z':
            let x = pending.pop()
            pending.pop()
            pending.push(Char(Int(x) + 1))
        while matched < b.len and head < pending.len and pending[head] == b[matched]:
            head += 1
            matched += 1
    print(if head == pending.len and matched == b.len: "YES" else: "NO")
