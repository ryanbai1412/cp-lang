// expect: exit 1
// rule: 09.3.8
// tool: stress --seed 1
// lacks: "error["
struct Counter:
    @cache hits: Int

    fn get(self) -> Int:
        self.hits += 1
        self.hits

fn reads(n: Int) -> Int:
    let c = Counter(0)
    for _ in 0..n: let _ = c.get()
    c.get()
@alt(reads)
fn reads_slow(n: Int) -> Int = n + 1
let n: 1..=8 = read()
print(reads(n))
