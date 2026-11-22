// expect: ok, output "1 -1"
// rule: 02.2.2
fn f(c: '+' | '-') -> Int = match c: '+' => 1, '-' => -1
print(f('+'), f('-'))
