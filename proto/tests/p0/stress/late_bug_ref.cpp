#include <cstdio>
int main() { int n; long long s = 0, x; scanf("%d", &n); for (int i = 0; i < n; ++i) { scanf("%lld", &x); s += x; } printf("%lld\n", s); }
