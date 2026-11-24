//! Messages cpl-lint's taste lints emit (`rules/taste/mod.rs`).

super::msgs! {
    ANSWER_SEARCH = "L0520", "a hand-rolled binary search: this is `{}`";
    PREFIX_SUM = "L0521", "a hand-rolled prefix sum into `{}`: use `Prefix(...)`";
    COPY_SORT_WRITEBACK = "L0522", "copies `{}[{}]`, mutates it, and writes it back: use `{}`";
    DISCARDED_RESULT = "L0502", "`let _ =` throws away what `{}` returns";
    SYMMETRIC_WRITE = "L0503", "one fact written twice, once for each order";
    SEARCH_LOOP_BREAKS = "L0501", "a search loop that sets `{}` and breaks";
    SEARCH_LOOP_FIRST = "L0501", "a search loop: only the first write to `{}` counts";
    PUSH_LOOP = "L0507", "a loop that only pushes onto `{}` is a comprehension";
    NONE_CHECK_UNWRAP = "L0508", "`{} != None` and then `{}!`: the check is said twice";
    FREE_FOLD = "L0509", "`{}` of one sequence reads as its method";
    TREE_ORDER = "L0511", "`{}` is the preorder array `{}`";
    DOMAIN_SORTED = "L0512", "use `{}.sorted(...)` instead of `{}.sort(...)`";
    FIND_WHERE = "L0512", "a predicate search is `.find(p)`, without `where:`";
    LOOP_FOLD = "L0504", "a loop that only folds `{}` through `{}` is one `fold`";
    LOOP_ACCUMULATE = "L0504", "a loop that only accumulates `{}` is one `{}`";
    INDEX_ONLY_LOOP = "L0505", "`{}` is only used as `{}[{}]`";
}
