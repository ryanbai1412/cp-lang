// expect: ok, output "3 2 1"
// rule: 07.2.2
// A domain's `.rev` walks it backwards, no parentheses (rebuild 7, H-REV-DOMAIN).
domain C = 1..=3
print([Int(c) for c in C.rev])
