// IOI 2022 "Rarest Insects": interactive; a machine reports the largest
// group of one type among the insects inside it. Find the rarest type's size.
//
// The judge calls `min_cardinality`; this file plays the sample grader: it
// reads n and each insect's type, answers the three machine calls (4e4 each)
// and prints the answer.
let n: 2..=2000 = read()
domain Insect = [n]
let kind = read<[Insect]0..=1e9>()

var calls: [3]Int = 0
fn call(op: 0..3):
    calls[op] += 1
    assert calls[op] <= 40000, "too many machine calls"

var in_machine: [Insect]Bool = false
var held: Counter<Int, Int> = Counter()          // insects inside, per type
var with_count: [0..=n]Int = 0                   // types inside, per group size
with_count[0] = n
var top = 0

fn move(i: Int, into: Bool):
    call(if into: 0 else: 1)
    let x = Insect(i)
    if in_machine[x] == into: return
    in_machine[x] = into
    let c = held[kind[x]]
    let d = if into: c + 1 else: c - 1
    with_count[c] -= 1
    with_count[d] += 1
    held[kind[x]] = d
    if into: top max= d
    elif top == c and with_count[c] == 0: top -= 1

fn move_inside(i: Int) = move(i, into: true)
fn move_outside(i: Int) = move(i, into: false)
fn press_button() -> Int:
    call(2)
    top

fn min_cardinality(n: Int) -> Int:
    domain Insect = [n]
    var inside: [Insect]Bool = false

    fn fill(cap: Int) -> List<Insect>:
        var added = List<Insect>()
        for i in Insect if not inside[i]:
            move_inside(Int(i))
            if press_button() > cap:
                move_outside(Int(i))
            else:
                inside[i] = true
                added.push(i)
        added

    let kinds = fill(1).len
    fn holds(cap: Int) -> Bool:
        let added = fill(cap)
        if inside.count(x => x) >= kinds * cap:          // kept fills stay valid below cap
            return true
        for i in added:
            move_outside(Int(i))
            inside[i] = false
        false

    (1..=n / kinds).last_true(holds)!

print(min_cardinality(n))
