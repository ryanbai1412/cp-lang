// A traced statement whose place is reached through a call, or whose
// value writes through such a place, runs on the VM as it does untraced;
// so does a generator's fed loop over such a value.
var hits: [0..4]Int = 0
var rows: List<List<Int>> = [[], [], [], []]
var stacks: List<List<Int>> = [[5, 3], [4]]
fn pick(i: Int) -> Int: i % 4

fn evens(k: Int) -> yields Int:
    for x in 0..k:
        if x % 2 == 0: yield x

fn chained(n: Int) -> yields Int:
    for x in evens(stacks[pick(n)].pop()!):
        yield x + n

for i in 0..6:
    hits[pick(i)] += i
    rows[pick(i)].push(i)
print(hits, rows)
for y in chained(0): print("y", y)
for y in chained(1): print("y", y)
print(stacks)
