// An optional that a test has shown to hold a value reads as that value:
// after `if x == none: return`, inside `if x != none:`, in the `else` of
// `if x == none:`, and on the right of `x == none or ...`.
fn find(xs: List<Int>, want: Int) -> ?Int:
    for i in 0..xs.len:
        if xs[i] == want: return i
    none

fn bump(x: ?Int) -> ?Int = if x == none: none else: x + 1

fn pick(x: ?Int, y: ?Int) -> Int:
    if x == none and y == none: return 0
    if x == none: return y ?? -1
    elif y == none: return x
    else: return x + y

fn old(x: ?Int) -> Int:
    if x != none: return x! + 1
    0

fn local(x: ?Int) -> Int:
    if x == none: return 0
    let add = y => x + y
    fn twice() -> Int = x * 2
    add(1) + twice()

fn rebound(x: ?Int) -> ?Int:
    var y = x
    if y != none:
        y = none
        return y
    y

fn fmt_opt(x: ?Int) -> Str = if let v = x: "{v}" else: "none"
let at = find([4, 5, 6], 5)
if at != none: print(at + 1)
if not (at == none): print(at * 10)
let gone = find([4, 5, 6], 7)
print(if gone == none: -1 else: gone + 1)
print(fmt_opt(bump(3)), fmt_opt(bump(none)), pick(none, none), pick(none, 2), pick(3, none), pick(3, 4))
print(old(4), old(none), local(5), local(none), fmt_opt(rebound(1)))
var cur: ?Int = 3
while cur != none and cur > 0:
    cur = if cur == 1: none else: cur - 1
print(fmt_opt(cur))
var total = 0
for want in [4, 6, 9]:
    let hit = find([4, 5, 6], want)
    if hit == none: continue
    total += hit
print(total)
var best: ?Int = none
for x in [4, 2, 7]:
    if best == none or x < best: best = x
print(fmt_opt(best))
