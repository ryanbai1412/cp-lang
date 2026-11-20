// A file-level @memo table lasts one test: `a` is new in each.
@multitest
let n: 1..=20 = read()
domain I = 1..=n
let a: [I]Int = read()

@memo
fn best(i: I) -> Int:
    if let j = i.next: max(a[i], a[i] + best(j)) else: a[i]
print(max(best(i) for i in I)!)
