// expect: R0401 (checked)
// `last_true` wants true then false; a false-then-true `ok` traps instead of answering the end.
print((0..10).last_true(x => x >= 4) ?? -1) // <- here
