// expect: E0314
// rule: 02.2.2
fn f(c: '+' | '-') -> Int = match c: '+' => 1 // <- here
print(f('+'))
