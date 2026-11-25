// expect: ok, output "body 0\nafter 0\nafter 2\nbody 4\n--\n0\n100\nafter 0\n2\n102\n--\n72\n5 -1"
// input: "9\n"
// Generators: the loop body runs inside the callee; leaving it stops the callee at once.
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0:
            yield i
            print("after {i}")
    print("evens done")
fn twice(n: Int) -> yields Int:
    for x in evens(n):
        yield x
        yield x + 100
    print("twice done")
fn first_over(xs: List<Int>, k: Int) -> Int:
    for x in xs:
        if x > k: return x
    0 - 1
let n: Int = read()
for b in evens(n):
    if b == 2: continue
    print("body {b}")
    if b == 4: break
print("--")
for t in twice(n):
    print(t)
    if t == 102: break
print("--")
var xs: List<Int> = [3, 1, 4, 1, 5, 9, 2, 6]
var s = 0
for x in xs:
    for y in xs:
        if y > x: break
        s += y
print(s)
print("{first_over(xs, 4)} {first_over(xs, 9)}")
