// The matched hand reader for nonnegative values: one fread, then a digit scan.
#include <bits/stdc++.h>
static char buf[1 << 27];
static size_t at;
static long long rd() {
    while (buf[at] == ' ' || buf[at] == '\n') at++;
    long long x = 0;
    while (buf[at] >= '0' && buf[at] <= '9') x = x * 10 + (buf[at++] ^ '0');
    return x;
}
int main() {
    if (!fread(buf, 1, sizeof buf - 1, stdin)) return 1;
    long long n = rd(), s = 0;
    std::vector<unsigned> a(n);
    for (auto& x : a) x = (unsigned)rd();
    for (unsigned x : a) s += x;
    printf("%lld\n", s);
}
