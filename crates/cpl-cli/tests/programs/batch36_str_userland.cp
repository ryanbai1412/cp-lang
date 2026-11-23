let st = "hello"
print(
    st.repeat(2),
    st.find("ll") ?? -1,
    st.find('z') ?? -1,
    yes(st.starts_with("he")),
    yes(st.ends_with("lo")),
    yes(st.ends_with("he")),
)
print("ab".repeat(0).len, yes("".starts_with("")), yes("a".starts_with("ab")))
