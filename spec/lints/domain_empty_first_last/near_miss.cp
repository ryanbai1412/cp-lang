// A domain the declared bounds keep non-empty.
let (n, k): (1..=10, 0..=10) = read()
domain V = 0..n
domain W = k..=k
print(V.first!.ord, V.last!.ord, W.first!.ord)
