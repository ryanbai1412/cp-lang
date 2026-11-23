// Writes the tracer sees only through a reference: each shows at its step.
var total = 0
var xs = [3, 1, 2]
for round in 0..2:
    fn note(mut count: Int, x: Int):
        count += x
    note(mut total, 2 + round)
    print(total)
xs.sort()
print(xs)
