#include <cstdio>
#include <vector>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<bool> comp(n + 1);
    int c = 0;
    for (long long i = 2; i <= n; i++) {
        if (comp[i]) continue;
        c++;
        for (long long j = i * i; j <= n; j += i) comp[j] = true;
    }
    printf("%d\n", c);
}
