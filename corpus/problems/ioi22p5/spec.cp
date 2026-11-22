// IOI 2022 "Rarest Insects": interactive; a machine reports the largest
// group of one type among the insects inside it. Find the rarest type's size.
// Greedily filling the machine with at most `cap` of each type takes exactly
// kinds * cap insects iff every type has at least cap, which is monotone in cap.
extern fn move_inside(i: Int)
extern fn move_outside(i: Int)
extern fn press_button() -> Int

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
        if inside.count(x => x) == kinds * cap:
            return true
        for i in added:
            move_outside(Int(i))
            inside[i] = false
        false

    (1..=n / kinds).last_true(holds)!
