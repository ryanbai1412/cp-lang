// Matched C++ for wrap.cp: the same digit DP with `(r + x) % d`.
#include <cstdio>
#include <vector>
int main() {
    const long long mod = 1000000007;
    const int d = 97;
    std::vector<long long> below(d, 0), next(d);
    below[0] = 1;
    for (int it = 0; it < 200000; it++) {
        std::fill(next.begin(), next.end(), 0);
        for (int r = 0; r < d; r++)
            for (int x = 0; x <= 9; x++) {
                long long& t = next[(r + x) % d];
                t += below[r];
                if (t >= mod) t -= mod;
            }
        below.swap(next);
    }
    printf("%lld\n", below[0]);
}
