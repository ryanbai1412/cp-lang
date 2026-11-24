# cheerio1s1 frictions

## Dream

```cp
let t = lights.fold((pos: 0, t: 0), (s, l) => wait(s.t + l.p - s.pos, l) as (l.p, _)).t
```

## Walking position held in a var

The previous light's position is carried in `pos`; a pairwise scan over `[0] + positions`
(`lights.with_prev`) would state the distance walked between lights directly.
