// expect: ok, output "1"
// rule: 05.10.13
struct C:
    @cache memo: List<Int>
    k: Int
let a = C(memo: [1], k: 3)
let b = C(memo: [2], k: 3)
print(if a == b: 1 else: 0)
