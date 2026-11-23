// expect: ok, output "3000000000"
// rule: 03.1.7
// A scan's sums are `Int`, never values of the element range, so they pass `1e9` (rebuild 8, H-PREFIX-ELEM-TYPE).
domain I = 0..3
let a: [I]1..=1e9 = [I] _ => 1e9
let pre = a.scan(+)
print(pre.all)
