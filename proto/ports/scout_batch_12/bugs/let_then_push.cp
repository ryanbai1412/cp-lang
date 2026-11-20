// A `let` that takes a list element apart, then a push to the same list:
// the names read freed memory. Built with proto/cpc and run under
// AddressSanitizer this reports heap-use-after-free; without it the
// program prints garbage. It should print 100 and nothing else.
// See L0 in research/friction-backlog-batch-12.md.
var xs: List<(Int, Int)> = [(1, 2)]
var next = 0
while next < xs.len:
    let (a, b) = xs[next]
    next += 1
    if xs.len < 100: xs.push((a + 1, b + 1))
    if a != b - 1: print(next, a, b)
print(xs.len)
