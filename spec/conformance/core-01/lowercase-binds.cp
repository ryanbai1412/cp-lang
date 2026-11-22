// expect: ok, output "7"
// rule: 01.3.3
let h: ?Int = Some(7)
match h:
    Some(big) => print(big)
    None => print(0)
