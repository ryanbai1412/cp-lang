// expect: ok, output "YES YES"
let p = manacher("abc")
print(yes(p.is_palindrome(1..1)), yes(p.is_palindrome(0..1)))
