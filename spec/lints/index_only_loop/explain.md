Iterating `for i in 0..xs.len` solely to read `xs[i]` introduces an unnecessary index variable. Iterate directly with `for x in xs:`.
