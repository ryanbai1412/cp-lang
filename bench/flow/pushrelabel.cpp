#include <bits/stdc++.h>
using namespace std;
#include "_pushrelabel.inc"
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
int main() {
    const int N = 5000; head.assign(N, {});
    for (int i = 0; i < 200000; i++) { int u = nxt(N), v = nxt(N); long long c = nxt(1000000) + 1; add(u, v, c); }
    printf("%lld\n", push_relabel(0, N - 1));
}
