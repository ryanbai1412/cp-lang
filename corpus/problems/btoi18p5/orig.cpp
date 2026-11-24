// BOI 2018 Genetics: the villain is the one sequence at distance exactly K from every other.
// Give each sequence a random weight w_i and, per column and letter, sum the weights of the
// sequences with that letter. Sequence i is the villain when the weighted sum of its distances,
// sum over columns of (W - S[col][its letter]), equals K * (W - w_i), checked mod 2^64.
#include <cstdio>
#include <cstdint>
#include <random>
#include <vector>

int main() {
    int n, m, k;
    if (std::scanf("%d %d %d", &n, &m, &k) != 3) return 0;
    std::vector<std::vector<unsigned char>> s(n, std::vector<unsigned char>(m));
    static char buf[5000];
    auto code = [](char c) { return c == 'A' ? 0 : c == 'C' ? 1 : c == 'G' ? 2 : 3; };
    for (int i = 0; i < n; i++) {
        std::scanf("%4100s", buf);
        for (int j = 0; j < m; j++) s[i][j] = code(buf[j]);
    }
    std::mt19937_64 rng(20180501);
    std::vector<uint64_t> w(n);
    uint64_t total = 0;
    for (auto& x : w) total += x = rng();
    std::vector<uint64_t> sum(4 * (size_t)m, 0);
    for (int i = 0; i < n; i++)
        for (int j = 0; j < m; j++) sum[4 * j + s[i][j]] += w[i];
    for (int i = 0; i < n; i++) {
        uint64_t got = 0;
        for (int j = 0; j < m; j++) got += total - sum[4 * j + s[i][j]];
        if (got == (uint64_t)k * (total - w[i])) {
            std::printf("%d\n", i + 1);
            return 0;
        }
    }
}
