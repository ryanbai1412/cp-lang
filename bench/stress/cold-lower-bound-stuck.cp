// The first position of a sorted array holding at least x, written cold for the Milestone 1 loop
// run, with a planted bug: `lo = mid` instead of `lo = mid + 1`, so once the range is two wide
// and its first value is below x the search never ends.
// class: complexity
// time-limit-ms: 300

let (n, x): (1..=8, 0..=10) = read()
let a: [0..n]0..=10 = read()
let b = sorted(a)
var lo = 0
var hi = n
while lo < hi:
    let mid = (lo + hi) / 2
    if b[mid] < x:
        lo = mid  // planted
    else:
        hi = mid
print(lo)

@alt(program)
fn linear():
    let b = sorted(a)
    var i = 0
    while i < n and b[i] < x: i += 1
    print(i)
