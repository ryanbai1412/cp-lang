// expect: ok, output "2"
fn lo<R: RangeEnds<Int>>(r: R) -> Int = r.start().ord
print(lo(2..5))
