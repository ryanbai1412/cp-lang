// expect: R0401 (checked)
let p = manacher("abc")
print(yes(p.is_palindrome(2..5))) // <- here
