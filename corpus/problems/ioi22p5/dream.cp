// IOI 2022 "Rarest Insects": interactive; a machine reports the largest
// group of one type among the insects inside it. Find the rarest type's size.
@grader fn move_inside(i: Int)
@grader fn move_outside(i: Int)
@grader fn press_button() -> Int

pub fn min_cardinality(n: Int) -> Int:
    domain Insect = [n]
    var inside: [Insect]Bool = false

    fn fill(cap: Int) -> List<Insect>:
        var added = List<Insect>()
        for i in Insect where not inside[i]:
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
        if inside.count() == kinds * cap:
            return true
        for i in added:
            move_outside(Int(i))
            inside[i] = false
        false

    (1..=n / kinds).last_true(holds)!
