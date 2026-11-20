let n: 1..=100 = read()
let q: 1..=100 = read()
domain V = 1..=n

struct Counter:
    cnt: [V]Int
    fn at(mut self, i: V) lends mut Int:
        mut self.cnt[i]

fn add_all(mut xs: [V]Int, d: Int):
    for v in V:
        xs[v] += d

var c = Counter([V] v => 0)
for _ in 0..q:
    let pos: V = read()
    let delta: Int = read()
    c.at(pos) += delta
add_all(mut c.cnt, 1)
print(c.cnt)
