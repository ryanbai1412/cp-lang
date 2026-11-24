// expect: ok, output "YES NO YES NO\nYES NO\n4"
// The `Char` tests are std bodies, with no import; `is_space` is the four ASCII spaces only.
let s = "ab1 Cd!"
print(yes('7'.is_digit), yes('a'.is_upper), yes('a'.is_lower), yes('['.is_alpha))
print(yes('\r'.is_space), yes(Char(11).is_space))
print(s.count(c => c.is_alpha))
