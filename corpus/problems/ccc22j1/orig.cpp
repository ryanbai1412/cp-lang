// CCC 2022 J1 Cupcake Party: 8 per regular box, 3 per small box, 28 students.
#include <cstdio>

int main() {
    long long r, s;
    std::scanf("%lld %lld", &r, &s);
    std::printf("%lld\n", 8 * r + 3 * s - 28);
}
