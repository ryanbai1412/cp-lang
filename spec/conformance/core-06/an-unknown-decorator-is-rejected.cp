// expect: E0107
// rule: 06.4.11
// R06.4.11: a decorator no chapter documents is E0107, so no later pass meets one.
@akaa(total) // <- here
fn sum3(a: Int, b: Int, c: Int) -> Int = a + b + c
print(sum3(1, 2, 3))
