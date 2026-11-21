#include <cstdint>
#include <iostream>
#include <vector>

using i64 = int64_t;

int main() {
    const i64 K = 4;
    i64 x = 112233445;
    std::vector<std::vector<i64>> pats(20000, std::vector<i64>(10));
    for (i64 i = 0; i < 20000; i++) {
        for (i64 j = 0; j < 10; j++) {
            x = (x * 1103515245 + 12345) & 0x7fffffff;
            pats[i][j] = (x >> 16) % K;
        }
    }
    std::vector<i64> kid(K, -1);
    std::vector<std::vector<i64>> ends(1);
    for (i64 i = 0; i < (i64)pats.size(); i++) {
        i64 at = 0;
        for (i64 a : pats[i]) {
            i64 idx = at * K + a;
            if (kid[idx] == -1) {
                i64 nw = (i64)ends.size();
                kid.insert(kid.end(), K, -1);
                ends.emplace_back();
                kid[idx] = nw;
            }
            at = kid[idx];
        }
        ends[at].push_back(i);
    }
    i64 n = (i64)ends.size();
    std::vector<i64> go(kid.size());
    for (size_t i = 0; i < kid.size(); i++) go[i] = kid[i] == -1 ? 0 : kid[i];
    std::vector<i64> fail(n, 0), exit_link(n, -1), order = {0};
    order.reserve(n);
    for (size_t head = 0; head < order.size(); head++) {
        i64 s = order[head];
        for (i64 c = 0; c < K; c++) {
            i64 past = s == 0 ? 0 : go[fail[s] * K + c];
            i64 t = kid[s * K + c];
            if (t != -1) {
                fail[t] = past;
                exit_link[t] = ends[past].empty() ? exit_link[past] : past;
                order.push_back(t);
            } else {
                go[s * K + c] = past;
            }
        }
    }
    i64 st = 0, hits = 0;
    for (i64 i = 0; i < 500000; i++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        st = go[st * K + ((x >> 16) % K)];
        if (!ends[st].empty() || exit_link[st] != -1) hits++;
    }
    std::cout << n << " " << hits << "\n";
}
