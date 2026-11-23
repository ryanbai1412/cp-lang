let n: 1..=100 = read()
var seen: Set<Int> = Set()
fn mark(x: Int) -> Bool:
    if x in seen:
        return false
    seen.add(x)
    return true
for i in 0..n:
    let _ = mark(i % 7)  // expect: discarded_result
print(seen.len)
