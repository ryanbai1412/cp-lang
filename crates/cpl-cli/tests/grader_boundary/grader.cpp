// A souvenirs-shaped grader (IOI 2025): `long long` and `std::pair`
// cross both ways, with values past `int`.
#include "souvenirs.h"
#include <cstdio>

std::pair<std::vector<int>, long long> transaction(long long M) {
    return {{1}, M - 1};
}

int main() {
    buy_souvenirs(2, 1000000000000000LL);
    return 0;
}
