// DMOPC '21 Contest 9 P2 — merge equal neighbours upward, then peel a prefix.
// original: corpus/submissions/4563973.cpp
@multitest
let (a, b): (Str, Str) = read()

var pending: Deque<Char> = []
var matched = 0
for c in a:
    pending.push(c)
    // FRICTION F2: "last two equal" has no pattern; index from the back.
    while pending.len >= 2 and pending[pending.len - 2] == pending.last and pending.last != 'z':
        let x = pending.pop()!
        pending.pop()
        pending.push(x + 1)
    while matched < b.len and pending.first == b[matched]:
        pending.pop_front()
        matched += 1
print(if pending.is_empty and matched == b.len: "YES" else: "NO")
