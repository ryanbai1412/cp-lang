var xs = [1, 2, 99]
var ys = [5, 6, 7]
while let a = xs.last, let b = ys.last, a == 99:
    print("w1", a, b)
    xs.pop()
    ys.pop()
print(xs, ys)
while let a = xs.last, a > 1 or ys.len == 3, let b = ys.pop(), not (b < 6), a + b > 0:
    print("w2", a, b)
    xs.pop()
print(xs, ys)
var k = 0
while k < 5, let a = xs.last, a + k < 3:
    k += 1
print(k)
fn show(o: ?Int, p: ?Int):
    if let a = o, let b = p, a == 99: print("a99", b)
    else: print("else1")
    if let a = o, a == 1, let b = p, b == 3: print("a1 b3")
    elif let a = o, let b = p, a == 1 and b == 2, not (a > b): print("elif", a, b)
    elif o == none, let b = p: print("only b", b)
    else: print("else2")
    if let a = o, let b = p, a == 5 or b == 2, a < b: print("or", a, b)
    if let a = o, a == 1, a == 2: print("never")
    elif let a = o, a >= 1, a <= 99, a != 50: print("three tests", a)
    else: print("else3")
    let v = if let a = o, let b = p, a < b: a + b else: -1
    print(v)
for o in [none, some(1), some(99), some(50)]:
    for p in [none, some(2), some(3)]:
        show(o, p)
var calls = 0
fn tick() -> ?Int:
    calls += 1
    some(calls)
if let a = tick(), a > 5, let b = tick(): print("no", b)
print(calls)
print([a + b for o in [some(1), none, some(3)] if let a = o and let b = tick() and a < b])
