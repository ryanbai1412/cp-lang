// expect: ok, output "! 3"
// rule: 03.2.4

print("! {(1..=5).last_true(x => x <= 3)!}")
