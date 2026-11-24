// expect: ok, output "4294967295"
// input: "-2147483648 2147483647\n"
// A range of two 32-bit ends skips the length check (6ad9177de) and still gives the full span, which fits in an `Int`.
let lo: -2_147_483_648..=2_147_483_647 = read()
let hi: -2_147_483_648..=2_147_483_647 = read()
print((lo..hi).len)
