// expect: ok, output "5"
// engines: next
fn hi<R: RangeEnds<Int>>(r: R) -> Int = r.end().ord
print(hi(2..5))
