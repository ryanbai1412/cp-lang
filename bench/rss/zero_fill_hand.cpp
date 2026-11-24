#include <cstdio>
static unsigned short t[80000000];
int main() { int i; if (scanf("%d", &i) != 1) return 1; t[i] = 3; printf("%d\n", t[i]); }
