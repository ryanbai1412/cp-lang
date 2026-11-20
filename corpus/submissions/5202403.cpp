#include "bits/extc++.h"

std::vector<std::pair<std::pair<int, int>, std::pair<int, int>>> BuildPattern(
    int n) {
    assert(1 <= n && n <= 2);
    if (n == 1) {
        return {};
    } else {
        return {{{0, 0}, {0, 1}}};
    }
}

int GetCardNumber(
    std::vector<std::pair<std::pair<int, int>, std::pair<int, int>>> p) {
    assert(p.size() <= 1);
    return p.size() + 1;
}