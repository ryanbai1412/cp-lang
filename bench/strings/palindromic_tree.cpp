#include <cstdint>
#include <iostream>
#include <vector>

using i64 = int64_t;

int main() {
    const i64 n = 1000000, K = 4;
    std::vector<i64> s(n);
    i64 x = 135792468;
    for (i64 i = 0; i < n; i++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        s[i] = (x >> 16) & 3;
    }
    std::vector<i64> next(2 * K, -1);
    std::vector<i64> slink = {0, 0};
    std::vector<i64> plen = {-1, 0};
    std::vector<i64> diff = {0, 0};
    std::vector<i64> series = {0, 0};
    std::vector<i64> cnt = {0, 0};
    std::vector<i64> chars;
    chars.reserve(n);
    i64 last = 1;
    for (i64 c : s) {
        i64 pos = (i64)chars.size();
        chars.push_back(c);
        i64 cur = last;
        while (true) {
            i64 l = plen[cur];
            if (pos - 1 - l >= 0 && chars[pos - 1 - l] == c) break;
            cur = slink[cur];
        }
        if (next[cur * K + c] == -1) {
            i64 id = (i64)plen.size();
            next.insert(next.end(), K, -1);
            i64 new_len = plen[cur] + 2;
            plen.push_back(new_len);
            i64 link_st = 1;
            if (new_len > 1) {
                i64 p = slink[cur];
                while (true) {
                    i64 l = plen[p];
                    if (pos - 1 - l >= 0 && chars[pos - 1 - l] == c) break;
                    p = slink[p];
                }
                link_st = next[p * K + c];
            }
            slink.push_back(link_st);
            i64 d = new_len - plen[link_st];
            diff.push_back(d);
            series.push_back(d == diff[link_st] ? series[link_st] : link_st);
            cnt.push_back(0);
            next[cur * K + c] = id;
        }
        last = next[cur * K + c];
        cnt[last] += 1;
    }
    for (i64 i = (i64)cnt.size() - 1; i >= 2; i--) {
        cnt[slink[i]] += cnt[i];
    }
    i64 ans = (i64)plen.size() - 2;
    for (i64 i = 2; i < (i64)cnt.size(); i++) {
        ans += cnt[i] * plen[i];
    }
    std::cout << ans << "\n";
}
