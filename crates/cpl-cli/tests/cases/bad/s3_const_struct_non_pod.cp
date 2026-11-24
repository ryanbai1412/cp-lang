// error: expected a name after `const`, found `struct`
// `const` names one compile-time scalar; a struct of settings is a `let` of a record, so there is no `const struct`.
const struct BadCfg:
    xs: List<Int>
