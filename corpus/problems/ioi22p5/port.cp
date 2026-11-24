// Source: IOI 2022 Day 2 Rarest Insects, https://dmoj.ca/problem/ioi22p5
// Problem: interactive: N <= 2000 insects of hidden types. Operations: move an insect into or out
//   of a machine, or press its button, which tells the size of the largest type group inside;
//   at most 4e4 of each, scored by the most of any kind. Return the size of the rarest type.
// Solution: in a shuffled order, keep each insect that leaves the button at 1: that gives one of
//   each of the k types. Then binary-search the answer t in 1..=N / k: add the outside insects
//   while the button stays <= t, stopping at t * k inside. Reaching t * k means each type has
//   >= t: keep them and drop the rejected ones from the pool; else undo and search below t
//   among the ones just added. O(N log N) operations.
// original: corpus/submissions/4786085.cpp
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
