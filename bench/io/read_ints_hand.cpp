// The matched hand reader: the whole input in one fread, then a digit scan.
#include <bits/stdc++.h>
static char buf[1 << 27];
static size_t at;
static long long rd() {
    while (buf[at] == ' ' || buf[at] == '\n') at++;
    bool neg = buf[at] == '-';
    if (neg) at++;
    long long x = 0;
    while (buf[at] >= '0' && buf[at] <= '9') x = x * 10 + (buf[at++] - '0');
    return neg ? -x : x;
}
int main() {
    if (!fread(buf, 1, sizeof buf - 1, stdin)) return 1;
    long long n = rd(), s = 0;
    for (long long i = 0; i < n; i++) s ^= rd();
    printf("%lld\n", s);
}
