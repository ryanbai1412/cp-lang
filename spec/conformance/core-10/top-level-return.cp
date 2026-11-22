// expect: ok, output "1"
// rule: 10.1.2
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
print(1)
if zero(3) == 0:
    return
print(2)
