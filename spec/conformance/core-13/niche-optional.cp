// expect: ok, output "255 -1"
// rule: 13.2.5
let xs: List<?0..=255> = [Some(255), None]
print(xs, none: -1)
