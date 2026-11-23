// here: 4
// Python's `range(n - 1, -1, -1)` (self-run mistakes): the fix was `(n - 1..-1).step(-1)`, which traps.
let n = 3
for i in range(n - 1, -1, -1):
    print(i)
