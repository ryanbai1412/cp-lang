// expect: ok, output "20 4"
// Generator writes a global counter; the body only reads it. Common "progress counter" shape.
var steps = 0
fn gen(n: Int) -> yields Int:
    for i in 0..n:
        steps += 1
        yield i
var s = 0
for v in gen(4): s += v * steps
print(s, steps)
