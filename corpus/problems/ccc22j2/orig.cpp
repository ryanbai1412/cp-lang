// CCC 2022 J2 Fergusonball Ratings: 5 stars a point, -3 a foul; count ratings above 40, + if all are.
#include <cstdio>

int main() {
    long long n, good = 0;
    std::scanf("%lld", &n);
    for (long long i = 0; i < n; i++) {
        long long p, f;
        std::scanf("%lld %lld", &p, &f);
        if (5 * p - 3 * f > 40) good++;
    }
    std::printf("%lld%s\n", good, good == n ? "+" : "");
}
