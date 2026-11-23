// expect: E0602 at 4
// rule: 05.7.4 05.9.1
// A walk over a view of `a` (chunks, windows, runs) lends `a` like `for x in a`; a push here frees the buffer the chunk reads (heap-use-after-free in C++).
var a = [3, 1, 2, 4]
for c in a.chunks(2):
    a.push(0)
    print(c.first!)
