// expect: ok, output "YES NO"
let p = manacher("ab")
print(yes(p.is_palindrome(1..1)), yes(p.is_palindrome(0..2)))
