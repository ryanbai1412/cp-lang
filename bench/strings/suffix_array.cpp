#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>

using i64 = int64_t;

static void induced_sort(
    const std::vector<i64>& vec,
    const std::vector<i64>& counts,
    const std::vector<uint8_t>& is_l,
    const std::vector<i64>& lms,
    std::vector<i64>& sa
) {
    i64 n = (i64)vec.size(), alpha = (i64)counts.size();
    std::vector<i64> bkt(alpha, 0);
    i64 sum = 0;
    for (i64 c = 0; c < alpha; c++) { sum += counts[c]; bkt[c] = sum; }
    std::fill(sa.begin(), sa.end(), -1);
    for (i64 k = (i64)lms.size() - 1; k >= 0; k--) {
        i64 pos = lms[k], c = vec[pos];
        sa[--bkt[c]] = pos;
    }
    sum = 0;
    for (i64 c = 0; c < alpha; c++) { bkt[c] = sum; sum += counts[c]; }
    for (i64 k = 0; k < n; k++) {
        i64 p = sa[k] - 1;
        if (p >= 0 && is_l[p]) sa[bkt[vec[p]]++] = p;
    }
    sum = 0;
    for (i64 c = 0; c < alpha; c++) { sum += counts[c]; bkt[c] = sum; }
    for (i64 k = n - 1; k >= 1; k--) {
        i64 p = sa[k] - 1;
        if (p >= 0 && !is_l[p]) sa[--bkt[vec[p]]] = p;
    }
}

static std::vector<i64> sa_is(const std::vector<i64>& vec, i64 alpha) {
    i64 n = (i64)vec.size();
    std::vector<i64> sa(n, 0);
    if (n <= 1) return sa;
    std::vector<i64> counts(alpha, 0);
    for (i64 c : vec) counts[c]++;
    std::vector<uint8_t> is_l(n, 0);
    std::vector<i64> lms;
    for (i64 i = n - 2; i >= 0; i--) {
        is_l[i] = vec[i] > vec[i + 1] || (vec[i] == vec[i + 1] && is_l[i + 1]);
        if (is_l[i] && !is_l[i + 1]) lms.push_back(i + 1);
    }
    std::reverse(lms.begin(), lms.end());
    induced_sort(vec, counts, is_l, lms, sa);
    i64 m = (i64)lms.size();
    std::vector<i64> sorted_lms(m, 0);
    i64 k_lms = 0;
    for (i64 x : sa) {
        if (x >= 1 && !is_l[x] && is_l[x - 1]) sorted_lms[k_lms++] = x;
    }
    i64 name = 0;
    sa[n - 1] = 0;
    for (i64 k = 1; k < m; k++) {
        i64 i = sorted_lms[k - 1], j = sorted_lms[k];
        bool diff = vec[i] != vec[j];
        if (!diff) {
            i64 a = i + 1, b = j + 1;
            while (true) {
                if (vec[a] != vec[b]) { diff = true; break; }
                bool la = !is_l[a] && is_l[a - 1];
                bool lb = !is_l[b] && is_l[b - 1];
                if (la || lb) { diff = !(la && lb); break; }
                a++; b++;
            }
        }
        if (diff) name++;
        sa[j] = name;
    }
    if (name + 1 < m) {
        std::vector<i64> lms_vec(m, 0);
        for (i64 i = 0; i < m; i++) lms_vec[i] = sa[lms[i]];
        auto lms_sa = sa_is(lms_vec, name + 1);
        for (i64 i = 0; i < m; i++) sorted_lms[i] = lms[lms_sa[i]];
    }
    induced_sort(vec, counts, is_l, sorted_lms, sa);
    return sa;
}

int main() {
    const i64 n = 200000;
    std::vector<i64> s(n, 0);
    i64 x = 135792468;
    for (i64 i = 0; i < n; i++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        s[i] = (x >> 16) % 4;
    }
    std::vector<i64> uniq = s;
    std::sort(uniq.begin(), uniq.end());
    uniq.erase(std::unique(uniq.begin(), uniq.end()), uniq.end());
    std::vector<i64> vec(n + 1, 0);
    for (i64 i = 0; i < n; i++) {
        vec[i] = (i64)(std::lower_bound(uniq.begin(), uniq.end(), s[i]) - uniq.begin()) + 1;
    }
    auto raw = sa_is(vec, (i64)uniq.size() + 1);
    std::vector<i64> sa(raw.begin() + 1, raw.end());
    std::vector<i64> rank(n, 0), lcp(n - 1, 0);
    for (i64 j = 0; j < n; j++) rank[sa[j]] = j;
    i64 k = 0;
    for (i64 i = 0; i < n; i++) {
        if (rank[i] == n - 1) { k = 0; continue; }
        i64 j = sa[rank[i] + 1];
        while (i + k < n && j + k < n && s[i + k] == s[j + k]) k++;
        lcp[rank[i]] = k;
        if (k > 0) k--;
    }
    i64 m = n - 1;
    i64 lg = 1;
    while ((1LL << lg) <= m) lg++;
    std::vector<i64> st(lg * m);
    for (i64 i = 0; i < m; i++) st[i] = lcp[i];
    for (i64 b = 1; b < lg; b++) {
        i64 step = 1LL << (b - 1);
        i64 lim = m - (1LL << b) + 1;
        for (i64 i = 0; i < lim; i++) {
            st[b * m + i] = std::min(st[(b - 1) * m + i], st[(b - 1) * m + i + step]);
        }
    }
    i64 ans = n * (n + 1) / 2;
    for (i64 v : lcp) ans -= v;
    for (i64 q = 0; q < 200000; q++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        i64 a = (x >> 8) % n;
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        i64 b = (x >> 8) % n;
        if (a == b) { ans += n - a; continue; }
        i64 ra = rank[a], rb = rank[b];
        if (ra > rb) std::swap(ra, rb);
        i64 len = rb - ra;
        i64 lv = 63 - __builtin_clzll((uint64_t)len);
        ans += std::min(st[lv * m + ra], st[lv * m + rb - (1LL << lv)]);
    }
    std::cout << ans << "\n";
}
