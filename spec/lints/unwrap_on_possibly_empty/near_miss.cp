// A list the declared bounds keep non-empty, and one whose length is tested.
let n: 1..=10 = read()
let a: [0..n]Int = read()
print(a.max()!, a.first!)
let m: 0..=10 = read()
let b: [0..m]Int = read()
if m > 0:
    print(b.min()!)
let q: 1..=5 = read()
for _ in 0..q:
    // A slice whose declared ends keep it non-empty, and one tested first.
    let l: 0..n = read()
    let r: l + 1..=n = read()
    print(a[l..r].max()!)
    let (x, y): (0..n, 0..=n) = read()
    if x < y:
        print(a[x..y].first!)
