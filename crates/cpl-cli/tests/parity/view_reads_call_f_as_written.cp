// A `first`, `last` or `len` of `xs.map(f)` or `xs.filter(p)` builds no
// list, and `f` runs as often and in the order the interpreter runs it:
// an `f` that writes or prints runs on every element, even for `first`.
var calls = 0
fn tap(x: Int) -> Int:
    calls += 1
    x * 10
fn loud(x: Int) -> Bool:
    print("at", x)
    x > 1
let xs: List<Int> = read()
print(xs.map(tap).first ?? -1, calls)
print(xs.map(x => tap(x)).first ?? -1, calls)
print(xs.filter(loud).first ?? -1)
print(xs.map(x => tap(x) + 1).last ?? -1, calls)
print(xs.filter(x => tap(x) > 20).count(), calls)
print(yes(xs.map(x => tap(x)).any(x => x > 15)), calls)
print(yes(xs.map(x => tap(x)).all(x => x < 15)), calls)
print([y + 1 for y in xs.map(x => tap(x))].len, calls)
print(xs.map(x => tap(x)).sum(), calls)
print(xs.items.len, "abc".items.last ?? (0, 'z'))
