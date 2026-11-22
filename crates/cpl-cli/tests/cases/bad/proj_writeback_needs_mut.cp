// error: code after the `lend` changes `self`, so `at` takes `mut self`
let n: 2..=9 = read()
domain V = 1..=n
struct Tally:
    cnt: [V]Int
    distinct: Int

    fn at(self, v: V) lends Int:
        lend cnt[v]
        if cnt[v] != 0: distinct += 1

let zero: [V]Int = 0
let t = Tally(zero, 0)
print(t.at(V.first))
