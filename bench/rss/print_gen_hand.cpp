#include <cstdio>
int main() { int n; if (scanf("%d", &n) != 1) return 1; for (int i = 1; i <= n; i++) printf(i > 1 ? " %d" : "%d", i); printf("\n"); }
