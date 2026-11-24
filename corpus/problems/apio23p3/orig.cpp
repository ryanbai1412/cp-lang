// APIO 2023 P3 Alice, Bob, and Circuit as the sample grader's stdin adaptation: n members (name
// and favourite number), then m letters (sender, recipient); print each member's result, the sum
// of the favourite numbers of the senders of the letters they received, mod 65536. That is the
// sample grader's output for any correct alice/bob/circuit, computed here directly.
#include <cstdio>
#include <map>
#include <string>
#include <vector>

int main() {
    int n, m;
    if (std::scanf("%d %d", &n, &m) != 2) return 0;
    std::map<std::string, int> id;
    std::vector<unsigned> num(n), res(n, 0);
    char a[8], b[8];
    for (int i = 0; i < n; i++) {
        std::scanf("%7s %u", a, &num[i]);
        id[a] = i;
    }
    for (int i = 0; i < m; i++) {
        std::scanf("%7s %7s", a, b);
        res[id[b]] = (res[id[b]] + num[id[a]]) & 65535;
    }
    for (int i = 0; i < n; i++) std::printf("%u\n", res[i]);
}
