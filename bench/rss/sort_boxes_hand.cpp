#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<std::pair<long long, long long>> boxes(1 << n);
    for (long long i = 0; i < (1 << n); i++) boxes[i] = {i * 7919 % 1000, i * 104729 % 1000003};
    std::sort(boxes.begin(), boxes.end());
    printf("%lld\n", boxes[0].second + boxes[(1 << n) - 1].second);
}
