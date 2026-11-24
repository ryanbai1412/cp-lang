// A `break` or `continue` goes to the loop it names, or the innermost one:
// out of nested loops, out of a `while` in a fed loop's body, and out of
// the fed loop itself, the same way in `cpl run` as in the C++.
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i

var hits = 0
'outer: for r in 0..5:
    for c in 0..5:
        if c > r: continue 'outer
        if r * c == 6: break 'outer
        hits += 1
print(hits)

var total = 0
'scan: for x in evens(20):
    var k = 0
    while true:
        k += 1
        if k > x: break
        if k == 7: continue 'scan
        if x == 16: break 'scan
        if k % 2 == 0: continue
        total += k
print(total)
