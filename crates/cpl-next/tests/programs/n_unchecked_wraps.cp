// expect: ok, output "0 -2 9223372036854775806\n12\n3"
// rule: R09.4.3
// `unchecked:` turns the overflow check off: `Int` `+`, `-` and `*` wrap inside it, through a fn
// it calls none of that.
let big = 4611686018427387904
fn mix(h: Int, x: Int) -> Int:
    var r = 0
    unchecked:
        r = h * 31 + x
    r
var s = 0
unchecked:
    let top = 9223372036854775807
    print(big * 4, top + top, 0 - top - 1 + 1 - 1 + top + top)
    s = mix(0, 12)
print(s)
unchecked:
    if s > 0:
        print(1 + 2)
