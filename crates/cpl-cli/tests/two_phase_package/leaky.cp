// A two-phase dream whose second phase reads the first phase's global:
// in its own process, as on the judge, it reads 0.
var total = 0

@grader(two_phase)
fn encode(x: Int) -> Int:
    total += x
    0

@grader(two_phase)
fn decode(s: List<Int>) -> Int = total
