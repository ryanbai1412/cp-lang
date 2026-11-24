# cheerio1s1 frictions

## Dream

```cp
let t = lights.fold((pos: 0, t: 0), (s, l) => wait(s.t + l.p - s.pos, l) as (l.p, _)).t
```

## Walking position held in a var

The previous light's position is carried in `pos`; a pairwise scan over `[0] + positions`
(`lights.with_prev`) would state the distance walked between lights directly.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let ps = [2, 5, 9]
for (prev, p) in ps.with_prev: print(p - (prev ?? 0))
```

```text
2
3
4
```

**Status:** fixed: `with_prev` yields `(?T, T)`, so the walk is `for (prev, l) in lights.with_prev` and `pos` goes.
