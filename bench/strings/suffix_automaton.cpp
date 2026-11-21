#include <cstdint>
#include <iostream>
#include <vector>

using i64 = int64_t;

int main() {
    const i64 n = 200000, K = 4;
    std::vector<i64> s(n);
    i64 x = 246813579;
    for (i64 i = 0; i < n; i++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        s[i] = (x >> 16) % K;
    }
    std::vector<i64> links = {-1}, max_len = {0}, occ = {0};
    std::vector<i64> next(K, -1);
    i64 last = 0;
    for (i64 c : s) {
        i64 cur = (i64)links.size();
        links.push_back(0);
        max_len.push_back(max_len[last] + 1);
        occ.push_back(1);
        next.insert(next.end(), K, -1);
        i64 p = last;
        while (p != -1 && next[p * K + c] == -1) {
            next[p * K + c] = cur;
            p = links[p];
        }
        if (p == -1) {
            links[cur] = 0;
        } else {
            i64 q = next[p * K + c];
            if (max_len[p] + 1 == max_len[q]) {
                links[cur] = q;
            } else {
                i64 clone = (i64)links.size();
                links.push_back(links[q]);
                max_len.push_back(max_len[p] + 1);
                occ.push_back(0);
                next.resize((clone + 1) * K);
                for (i64 d = 0; d < K; d++) next[clone * K + d] = next[q * K + d];
                while (p != -1 && next[p * K + c] == q) {
                    next[p * K + c] = clone;
                    p = links[p];
                }
                links[q] = clone;
                links[cur] = clone;
            }
        }
        last = cur;
    }
    i64 m = (i64)links.size();
    i64 max_l = 0;
    for (i64 l : max_len) if (l > max_l) max_l = l;
    std::vector<i64> cnt(max_l + 1, 0);
    for (i64 l : max_len) cnt[l]++;
    for (i64 i = 1; i <= max_l; i++) cnt[i] += cnt[i - 1];
    std::vector<i64> order(m, 0);
    for (i64 v = 0; v < m; v++) order[--cnt[max_len[v]]] = v;
    for (i64 i = m - 1; i >= 1; i--) {
        i64 v = order[i];
        occ[links[v]] += occ[v];
    }
    i64 ans = 0;
    for (i64 v = 1; v < m; v++) ans += max_len[v] - max_len[links[v]];
    for (i64 v : occ) ans += v;
    std::cout << ans << "\n";
}
