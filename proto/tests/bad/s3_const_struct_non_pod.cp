// error: plain scalars only
// Synthesis #3 Leap 4: `const struct` only allows POD scalar fields (no List/Array/etc.).
const struct BadCfg:
    xs: List<Int>
