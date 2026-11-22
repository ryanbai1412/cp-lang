// expect: ok, output "a 7 A ? 7 0\nYES"
use std.strings.{lower, upper, digit, is_digit, is_lower, is_upper, is_alpha, is_alnum, is_space}
print('A'.lower, '7'.lower, 'a'.upper, '?'.upper, '7'.digit, '0'.digit)
print(yes('5'.is_digit and not 'x'.is_digit and 'q'.is_lower and not 'Q'.is_lower and 'Q'.is_upper and not 'q'.is_upper and 'Z'.is_alpha and not '_'.is_alpha and '9'.is_alnum and not ' '.is_alnum and '\t'.is_space and not 'x'.is_space))
