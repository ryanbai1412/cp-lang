// BF 3: the smallest prime >= N, by trial division up to sqrt.
#include <cstdio>

static bool prime(long long n) {
    if (n < 2) return false;
    for (long long d = 2; d * d <= n; d++)
        if (n % d == 0) return false;
    return true;
}

int main() {
    long long n;
    std::scanf("%lld", &n);
    while (!prime(n)) n++;
    std::printf("%lld\n", n);
}
