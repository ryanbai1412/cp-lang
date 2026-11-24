// A loop fed by a `yields` fn whose body jumps further out, to a loop
// around it or a `do`'s end: the fed fn stops as on a `break`, then the
// jump goes on.
fn upto(n: Int) -> yields Int:
    for i in 0..n:
        yield i

var hits = 0
'rows: for r in 0..4:
    for c in upto(5):
        if c == r: continue 'rows
        if r == 3 and c == 1: break 'rows
        hits += 10 * r + c
print(hits)
