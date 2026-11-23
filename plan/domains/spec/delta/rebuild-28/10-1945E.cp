// CF 1945E Binary Search: run the search on the unsorted p; swapping x into the final l is
// always enough. Output is a count line followed by one line per swap.
@check fn ok() -> Bool:
    let n: Int = read()
    let x: Int = read()
    var p = read<[n]Int>()
    let k: 0..=2 = read_output()
    repeat k:
        let (i, j): (1..=n, 1..=n) = read_output()
        p.swap_at(i - 1, j - 1)
    var (l, r) = (1, n + 1)
    while r - l > 1:
        let mid = (l + r) / 2
        if p[mid - 1] <= x:
            l = mid
        else:
            r = mid
    p[l - 1] == x

@!multitest
let n: 1..=2e5 = read()
let x: 1..=n = read()
let p = read<[n]1..=n>()
let at = p.keys.find(i => p[i] == x)!
var (l, r) = (1, n + 1)
while r - l > 1:
    let mid = (l + r) / 2
    if p[mid - 1] <= x:
        l = mid
    else:
        r = mid
print(1)
print(l, at + 1)
