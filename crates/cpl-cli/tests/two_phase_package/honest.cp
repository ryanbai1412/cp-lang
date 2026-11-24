// A two-phase dream: the second phase decodes only what the first sent.
@grader(two_phase)
fn encode(x: Int) -> Int = x

@grader(two_phase)
fn decode(s: List<Int>) -> Int = s.sum()
