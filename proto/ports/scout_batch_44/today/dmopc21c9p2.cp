// DMOPC '21 C9 P2 (CP Lang 0.2.2): merge equal adjacent characters while
// matching prefix against target `b` using `std.deque.Deque`.
// original: corpus/submissions/4563973.cpp
use std.deque.{Deque}

@multitest
let (a, b): (Str, Str) = read()
var pending = Deque<Char>.new()
var matched = 0
for c in a:
    pending.push(c)
    while pending.len >= 2 and pending[pending.len - 1] == pending[pending.len - 2] and pending[pending.len - 1] != 'z':
        let x = pending.pop()!
        pending.pop()
        pending.push(Char(Int(x) + 1))
    while matched < b.len and not pending.is_empty and pending[0] == b[matched]:
        pending.pop_front()
        matched += 1
print(if pending.is_empty and matched == b.len: "YES" else: "NO")
