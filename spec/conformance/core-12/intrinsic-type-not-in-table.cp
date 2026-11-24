// expect: E0210
// rule: 12.14.6
// A type's storage comes from the table too: an id it doesn't have is an error, not an empty struct.
pub struct Bag<K> = @intrinsic("bag")  // <- here
print(1)
