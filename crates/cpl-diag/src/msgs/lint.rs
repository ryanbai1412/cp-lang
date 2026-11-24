//! Messages cpl-lint's taste lints emit (`rules/taste/mod.rs`).

super::msgs! {
    ANSWER_SEARCH = "L0520", "a hand-rolled binary search: this is `{}`";
    DISCARDED_RESULT = "L0502", "`let _ =` throws away what `{}` returns";
    SEARCH_LOOP_BREAKS = "L0501", "a search loop that sets `{}` and breaks";
    SEARCH_LOOP_FIRST = "L0501", "a search loop: only the first write to `{}` counts";
    PUSH_LOOP = "L0507", "a loop that only pushes onto `{}` is a comprehension";
    DOMAIN_SORTED = "L0512", "use `{}.sort(...)` instead of `{}.sorted(...)`";
    FIND_WHERE = "L0512", "a predicate search is `.find(p)`, without `where:`";
}
