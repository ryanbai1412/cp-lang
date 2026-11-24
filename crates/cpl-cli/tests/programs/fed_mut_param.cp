// A fed loop's body updates its fn's `mut` param, traced or not.
fn upto(n: Int) -> yields Int:
    for i in 0..n: yield i

fn total_of(mut total: Int, n: Int) -> Int:
    for x in upto(n):
        total += x
    total

fn running(mut total: Int, n: Int) -> yields Int:
    for x in upto(n):
        total += x
        yield total

var t = 100
print(total_of(mut t, 4), t)
for s in running(mut t, 3): print(s)
print(t)
