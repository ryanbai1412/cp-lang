let st = "hello"
print(
    st.repeat(2),
    st.find("ll"),
    st.find('z'),
    st.starts_with("he"),
    st.ends_with("lo"),
    st.ends_with("he"),
)
print("ab".repeat(0).len, "".starts_with(""), "a".starts_with("ab"))
