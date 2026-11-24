# Friction report: modinv

## Dream

```cp
let (x, m): (U64, U64) = read()
print(x.inverse_mod(m)!)
```

## No unsigned modular inverse in std

`Mod<P>` needs a compile-time modulus that fits `Int`, and M goes up to 1e19, so the port
runs extended Euclid on `U64` itself with a sign flag. A std `inverse_mod` over `U64` (or an
`I128` extended gcd) would turn it into one call.

## Already at bar

The v0.2 port already had typed `U64` reads, the bound assert and no sentinels, so the re-port
adds nothing but tests.toml. 30 random cases agree with the v0.2 port and 30 with the
accepted C++, and the corpus tests pass.
