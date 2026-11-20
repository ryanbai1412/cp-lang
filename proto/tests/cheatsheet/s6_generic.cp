// cp: 0.1 bea787c+dirty
let n: 1..=10 = read()
domain V = 1..=n
let a: [V]Int = read()
let v = V(2)
fn total<D>(a: [D]Int) -> Int:
    return sum(a[i] for i in D)
print(total(a))
