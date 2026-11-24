// In-place verbs on a `mut` slice of a local and of a top-level list
// change the cells the slice covers and no others.
fn verbs() -> List<Int>:
    var a = [5, 1, 4, 2, 3, 9]
    a[0..5].sort()
    reverse(mut a[1..4])
    swap(mut a[0..1], mut a[4..5])
    a[2..4].fill(0)
    a
var g = [8, 6, 7, 5, 3, 0, 9]
g[2..6].sort()
reverse(mut g[..3])
print(verbs(), g)
