// R04.3.2: a cut has `c.next: ?V.cuts` and `c.r: ?V`. `cpl check` accepts this; `cpl run` rejects it.
domain I = [3]
let a: [I]Int = [10, 20, 30]
let c = I.cuts.first!
print(a[c.r!], Int(c.next!))
