// expect: R0401
// rule: std.seq.rekey
// `.rekey` with a key count unlike the cell count stops at std's `rekey` assert on both engines, though the call keeps its O(1) relabelling.
print([4, 5].rekey(10..13)) // <- here
