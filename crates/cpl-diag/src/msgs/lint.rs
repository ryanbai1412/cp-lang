//! Messages cpl-lint's taste lints emit (`rules/taste/mod.rs`).

super::msgs! {
    ANSWER_SEARCH = "L0520", "a hand-rolled binary search: this is `{}`";
    COPY_SORT_WRITEBACK = "L0522", "copies `{}[{}]`, mutates it, and writes it back: use `{}`";
    DISCARDED_RESULT = "L0502", "`let _ =` throws away what `{}` returns";
    PUSH_LOOP = "L0507", "a loop that only pushes onto `{}` is a comprehension";
    FREE_FOLD = "L0509", "`{}` of one sequence reads as its method";
    DOMAIN_SORTED = "L0512", "use `{}.sort(...)` instead of `{}.sorted(...)`";
    FIND_WHERE = "L0512", "a predicate search is `.find(p)`, without `where:`";
    LOOP_FOLD = "L0504", "a loop that only folds `{}` through `{}` is one `fold`";
    LOOP_ACCUMULATE = "L0504", "a loop that only accumulates `{}` is one `{}`";
}
