// error: must reach one `lend place`
let n: 2..=9 = read()
domain V = 1..=n
struct Bag:
    cnt: [V]Int
    fn at(self, i: V) lends mut Int:
        cnt[i]
