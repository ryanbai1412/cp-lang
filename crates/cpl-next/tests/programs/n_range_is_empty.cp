// expect: ok, output "1 0 0 1"
// rule: R04.4.12
// `r.is_empty` is a range's own getter, for literal, computed and dropped ranges.
fn bit(b: Bool) -> Int = if b: 1 else: 0
let n: Int = 3
let a = 5..n
let b = 0..n
let c = b.drop(2)
print(bit(a.is_empty), bit(b.is_empty), bit(c.is_empty), bit(b.drop(5).is_empty))
