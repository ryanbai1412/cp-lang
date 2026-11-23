// A list the declared bounds keep non-empty, and one whose length is tested.
let n: 1..=10 = read()
let a: [0..n]Int = read()
print(a.max()!, a.first!)
let m: 0..=10 = read()
let b: [0..m]Int = read()
if m > 0:
    print(b.min()!)
