// expect: ok, output "5"
fn hi<R: RangeEnds<Int>>(r: R) -> Int = r.end().ord
print(hi(2..5))
