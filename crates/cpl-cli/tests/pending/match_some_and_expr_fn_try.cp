// `match` on `?T` with `some(pat)` / `none`, and `?` inside expression-bodied
// `fn ... = if ...:` branches.
enum Parity = Even | Odd

fn classify(o: ?Parity) -> Int = match o:
    some(Odd) => 1
    some(Even) => 2
    none => 0

fn add_pair(o: ?(Int, Int)) -> Int = match o:
    some((a, b)) => a + b
    none => -1

fn walk(a: Int, b: Int, room: Int) -> ?List<Int> =
    if room == 0: none
    elif a < b: [0] + walk(b, a, room - 1)?
    elif a == b: if a == 1: some([1]) else: none
    else: [1] + walk(b, a - b, room - 1)?

print(classify(some(Parity.Odd)), classify(some(Parity.Even)), classify(none))
print(add_pair(some((3, 4))), add_pair(none))
let w1 = walk(3, 2, 10)
let w2 = walk(3, 2, 1)
print(w1 ?? [], w2 == none)
