// expect: ok, output "YES 255"
// rule: 10.2.4
let bad = "300".parse<0..=255>()
let good = "255".parse<0..=255>()
print(yes(bad == None), good!)
