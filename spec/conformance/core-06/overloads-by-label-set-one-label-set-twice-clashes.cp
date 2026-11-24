// expect: E0702
// rule: 06.1.6
// Two fns of one name and one label set are one signature twice (R06.1.6).
fn pick(xs: List<Int>; by: Int -> Int) -> Int = 1
fn pick(ys: List<Int>; by: Int -> Int) -> Int = 2 // <- here
