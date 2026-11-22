// expect: ok, output "15"
// rule: 07.12.3
print(iterate(1, x => if x < 8: Some(x * 2) else: None).sum())
