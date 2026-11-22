// `for x in f(..)` over a `yields` fn runs its body as the fn yields, with
// no list in the C++. `break`, `continue` and `return` in the body, loops
// inside it, and a second walk from inside the first all work as on a list.
var kid: [0..64]?Int
let n: Int = read()
for i in 1..n: kid[i] = if 2 * i < n: some(2 * i) else: none

fn upto(k: Int) -> yields Int:
    for i in 0..k: yield i

fn chain(from: Int) -> yields Int:
    var o = from
    yield o
    while true:
        if let next = kid[o]:
            yield next
            o = next
        else:
            return

fn pairs(k: Int) -> yields (Int, Str):
    for i in 0..k:
        if i == 2: continue
        yield (i, "v{i}")
    yield (100, "end")

fn first_over(k: Int, bar: Int) -> ?Int:
    for x in upto(k):
        if x * x > bar: return x
    none

fn names(k: Int) -> List<Str>:
    var out: List<Str> = []
    for (i, s) in pairs(k):
        if i == 1: continue
        if i == 100: return out + ["done"]
        out.push(s)
    out

fn nested(k: Int) -> Int:
    var total = 0
    for a in upto(k):
        for b in upto(a):
            if b == 3: break
            if (a + b) % 2 == 0: continue
            for c in 0..b:
                if c == 2: break
                total += 1
            if a * b == 42: return -total
            total += a * b
    total

fn skip(k: Int):
    for x in upto(k):
        if x == 3:
            print("stop at", x)
            return
        print("saw", x)

// The body changes what the fn reads: the list is made first.
var marks: [0..8]Int = 1
fn marked() -> yields Int:
    for i in 0..8:
        if marks[i] == 1: yield i
fn clears() -> Int:
    var seen = 0
    for i in marked():
        seen += 1
        if i + 1 < 8: marks[i + 1] = 0
    seen

print(first_over(10, 30), first_over(3, 30))
print(names(2), names(5))
print(nested(6), nested(9))
skip(2)
skip(6)
for col in chain(1):
    var value = 0
    for cell in chain(col): value += cell
    if col == 4: continue
    print(col, value)
    if col >= 8: break
print(clears(), marks)
var got = 0
for x in upto(5):
    for y in upto(x):
        if y == 2: break
        got += x * 10 + y
print(got)
for x in upto(4):
    if x == 2: return
    print("top", x)
print("unreachable")
