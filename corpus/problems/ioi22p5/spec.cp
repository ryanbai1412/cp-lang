// IOI 2022 "Rarest Insects": interactive; a machine reports the largest
// group of one type among the insects inside it. Find the rarest type's size.
// Greedily filling the machine with at most `cap` of each type takes exactly
// kinds * cap insects iff every type has at least cap, which is monotone in cap.
//
// The task is a grader function over three machine operations. Locally this
// reads the sample grader's input (N, then the N hidden types), plays the
// machine, and prints the answer; the sample grader's second line, the most
// calls of any one operation, is the task's score, not its answer.
let n: 2..=2000 = read()
domain Insect = [n]
let kind = read<[Insect]0..=1e9>()

let limit = 40000
var (ins, outs, presses) = (0, 0, 0)
var in_machine: [Insect]Bool = false
var held: Counter<Int, Int> = Counter()
var types_with: [0..=n]Int = 0
var top = 0

fn move_inside(i: Int):
    ins += 1
    assert ins <= limit, "too many calls to move_inside"
    let x = Insect(i)
    if in_machine[x]: return
    in_machine[x] = true
    let c = held[kind[x]]
    types_with[c] -= 1
    types_with[c + 1] += 1
    held[kind[x]] = c + 1
    top max= c + 1

fn move_outside(i: Int):
    outs += 1
    assert outs <= limit, "too many calls to move_outside"
    let x = Insect(i)
    if not in_machine[x]: return
    in_machine[x] = false
    let c = held[kind[x]]
    types_with[c] -= 1
    types_with[c - 1] += 1
    held[kind[x]] = c - 1
    if top == c and types_with[c] == 0: top -= 1

fn press_button() -> Int:
    presses += 1
    assert presses <= limit, "too many calls to press_button"
    top

@grader
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
        if inside.count(x => x) >= kinds * cap:
            return true
        for i in added:
            move_outside(Int(i))
            inside[i] = false
        false

    (1..=n / kinds).last_true(holds)!

print(min_cardinality(n))
