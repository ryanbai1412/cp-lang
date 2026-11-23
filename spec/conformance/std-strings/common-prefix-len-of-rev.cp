// expect: ok, output "4"
// rule: 07.3.1
// `common_prefix_len` only walks both sides, so a reversed view passes (rebuild round 29 H02).
print(common_prefix_len("abcx", "xcba".rev))
