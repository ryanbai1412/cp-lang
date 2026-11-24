//! C++ vocabulary: names of the C++ standard library that the C++ backend
//! writes and the C++-habit translator reads. They are C++'s names, not CP
//! Lang's, so a std fn with the same spelling (`fixed`, `map`) is unrelated
//! to them. Keeping them in this table means no compiler crate spells a std name.

/// The `std` class templates that a type a span names can be: `std::min<ll>`
/// is a fn, never a type.
pub const CLASS_TEMPLATES: &[&str] = &[
    "array",
    "bitset",
    "deque",
    "function",
    "map",
    "multiset",
    "optional",
    "pair",
    "set",
    "string",
    "tuple",
    "unordered_map",
    "unordered_set",
    "vector",
];

/// The `std` templates of types alone, which `template<class...T>using` can
/// stand for wherever they are written with their arguments.
pub const TYPE_TEMPLATES: &[&str] = &[
    "vector",
    "optional",
    "tuple",
    "pair",
    "set",
    "map",
    "multiset",
    "multimap",
    "deque",
    "function",
    "unordered_map",
    "unordered_set",
    "priority_queue",
];

/// The iostream manipulator in `cout << fixed << setprecision(9)`.
pub const FIXED: &str = "fixed";
