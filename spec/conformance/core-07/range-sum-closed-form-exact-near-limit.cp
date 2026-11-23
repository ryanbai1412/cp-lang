// expect: ok, output "-3000000000 9223372034707292160"
// rule: 07.9.6
// A closed-form range sum is exact when the sum fits, even when n times n would not.
print((-3_000_000_000..3_000_000_000).sum(), (1..=4_294_967_295).sum())
