// A projection may run code after its `lend`: the write-back, which runs when the caller is done
// with the place (plan/19 §3.4). Here it keeps `distinct`, the number of non-zero counts, right
// however the caller changes a count. This is what the library's Counter<K> is.
let n: 1..=9 = read()
domain V = 1..=n
let a: [n]V = read()

// counts that also know how many of them are not zero
struct Tally:
    cnt: [V]Int
    distinct: Int

    fn at(mut self, v: V) lends mut Int:
        if cnt[v] != 0: distinct -= 1
        lend mut cnt[v]
        if cnt[v] != 0: distinct += 1

fn bump(mut x: Int): x += 1

let zero: [V]Int = 0
var t = Tally(zero, 0)
for v in a:
    t.at(v) += 1
    let now = t.at(v)
    print(v, now, t.distinct)
t.at(V(1)) -= 1
print(t.distinct)
bump(mut t.at(V(4)))
let seen = t.at(V(4))
print(seen, t.distinct, t.cnt)
t.at(V(4)) = 0
print(t.distinct)
