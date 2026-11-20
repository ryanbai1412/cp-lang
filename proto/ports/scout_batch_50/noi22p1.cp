// NOI 2022 P1, strict majority of a concatenation of sequences under push, pop and join
// original: corpus/submissions/4800882.cpp (20 random probes; this port is exact)
let (n, q): (1..=5e5, 1..=5e5) = read()
domain Bit = 0..20                      // values are at most n + q < 2^20

struct Seq:
    items: Deque<Int> = []
    copies: HashMap<Int, Int> = HashMap()
    ones: [Bit]Int = 0                  // how many items have bit b set
    fn count(mut self, x: Int, d: Int):
        self.copies[x] += d
        for b in Bit: self.ones[b] += d * (x >> b.to_idx & 1)
    fn push(mut self, x: Int):
        self.items.push(x)
        self.count(x, 1)
    fn pop(mut self):
        self.count(self.items.pop()!, -1)

var seqs: List<Seq> = []
var store: [1..=n + q]Int = 0           // sequence label -> its Seq
for label in 1..=n:
    store[label] = seqs.push_id(Seq())
    let len: 0..=5e5 = read()
    for _ in 0..len: seqs[store[label]].push(read())

for _ in 0..q:
    let kind: 1..=4 = read()
    match kind:
        1 =>
            let (x, y): (1..=n + q, Int) = read()
            seqs[store[x]].push(y)
        2 =>
            let x: 1..=n + q = read()
            seqs[store[x]].pop()
        3 =>
            let m: 1..=5e5 = read()
            let picked: List<Int> = [store[read()] for _ in 0..m]
            let total = sum(seqs[s].items.len for s in picked)
            // each bit of a strict majority value is the majority of that bit
            let guess = sum(1 << b.to_idx for b in Bit if 2 * sum(seqs[s].ones[b] for s in picked) > total)
            let copies = sum(seqs[s].copies.get(guess) ?? 0 for s in picked)
            print(if 2 * copies > total: guess else: -1)
        4 =>
            let (x, y, z): (1..=n + q, 1..=n + q, 1..=n + q) = read()
            let (front, back) = (store[x], store[y])
            // the smaller side drains into the larger, at the matching end
            if seqs[front].items.len < seqs[back].items.len:
                for v in seqs[front].items.take().rev:
                    seqs[back].items.push_front(v)
                    seqs[back].count(v, 1)
                store[z] = back
            else:
                for v in seqs[back].items.take(): seqs[front].push(v)
                store[z] = front
