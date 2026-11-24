#include <cstdio>
#include <vector>
int main() {
    long long n, rounds, total = 0;
    if (scanf("%lld %lld", &n, &rounds) != 2) return 1;
    for (long long r = 0; r < rounds; r++) {
        std::vector<std::vector<long long>> rows(n);
        for (long long i = 0; i < n; i++) rows[(r * 1000 + i / 50) % n].push_back(i);
        for (auto& row : rows) if (!row.empty()) total += row[0] + (long long)row.size();
    }
    printf("%lld\n", total);
}
