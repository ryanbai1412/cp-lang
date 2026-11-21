#include <algorithm>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>

using i64 = int64_t;

static std::vector<i64> sa_is(const std::vector<i64>& s, i64 k) {
    i64 n = (i64)s.size();
    std::vector<i64> sa(n, -1);
    std::vector<uint8_t> is_s(n, 0);
    is_s[n - 1] = 1;
    for (i64 i = n - 2; i >= 0; i--)
        is_s[i] = (s[i] < s[i + 1]) || (s[i] == s[i + 1] && is_s[i + 1]);
    std::vector<i64> lms;
    std::vector<i64> lms_id(n, -1);
    for (i64 i = 1; i < n; i++) {
        if (is_s[i] && !is_s[i - 1]) {
            lms_id[i] = (i64)lms.size();
            lms.push_back(i);
        }
    }
    std::vector<i64> bucket(k, 0);
    for (i64 x : s) bucket[x]++;
    std::vector<i64> b_start(k), b_end(k);
    i64 sum = 0;
    for (i64 c = 0; c < k; c++) {
        b_start[c] = sum;
        sum += bucket[c];
        b_end[c] = sum;
    }
    auto induce = [&](const std::vector<i64>& order) {
        std::fill(sa.begin(), sa.end(), -1);
        std::vector<i64> ptr = b_end;
        for (i64 i = (i64)order.size() - 1; i >= 0; i--) {
            i64 p = order[i];
            sa[--ptr[s[p]]] = p;
        }
        ptr = b_start;
        for (i64 i = 0; i < n; i++) {
            i64 p = sa[i] - 1;
            if (p >= 0 && !is_s[p]) sa[ptr[s[p]]++] = p;
        }
        ptr = b_end;
        for (i64 i = n - 1; i >= 0; i--) {
            i64 p = sa[i] - 1;
            if (p >= 0 && is_s[p]) sa[--ptr[s[p]]] = p;
        }
    };
    induce(lms);
    i64 m = (i64)lms.size();
    if (m > 1) {
        std::vector<i64> sorted_lms;
        sorted_lms.reserve(m);
        for (i64 p : sa) if (lms_id[p] != -1) sorted_lms.push_back(p);
        std::vector<i64> reduced(m, 0);
        i64 name = 0;
        for (i64 i = 1; i < m; i++) {
            i64 a = sorted_lms[i - 1], b = sorted_lms[i];
            bool diff = false;
            for (i64 d = 0;; d++) {
                if (s[a + d] != s[b + d] || is_s[a + d] != is_s[b + d]) { diff = true; break; }
                if (d > 0 && (lms_id[a + d] != -1 || lms_id[b + d] != -1)) {
                    if (lms_id[a + d] == -1 || lms_id[b + d] == -1) diff = true;
                    break;
                }
            }
            if (diff) name++;
            reduced[lms_id[b]] = name;
        }
        std::vector<i64> ordered(m);
        if (name + 1 < m) {
            auto sub = sa_is(reduced, name + 1);
            for (i64 i = 0; i < m; i++) ordered[i] = lms[sub[i]];
        } else {
            for (i64 i = 0; i < m; i++) ordered[reduced[i]] = lms[i];
        }
        induce(ordered);
    }
    return sa;
}

struct SuffixArray {
    std::vector<i64> s, sa, rank, lcp, rmq;
    i64 rmq_n = 0;
    explicit SuffixArray(std::vector<i64> a) : s(std::move(a)) {
        i64 n = (i64)s.size();
        std::vector<i64> vals = s;
        std::sort(vals.begin(), vals.end());
        vals.erase(std::unique(vals.begin(), vals.end()), vals.end());
        std::vector<i64> enc(n + 1, 0);
        for (i64 i = 0; i < n; i++)
            enc[i] = (i64)(std::lower_bound(vals.begin(), vals.end(), s[i]) - vals.begin()) + 1;
        auto full = sa_is(enc, (i64)vals.size() + 1);
        sa.assign(full.begin() + 1, full.end());
        rank.assign(n, 0);
        for (i64 j = 0; j < n; j++) rank[sa[j]] = j;
        lcp.assign(n > 0 ? n - 1 : 0, 0);
        i64 k = 0;
        for (i64 i = 0; i < n; i++) {
            i64 r = rank[i];
            if (r == n - 1) { k = 0; continue; }
            i64 j = sa[r + 1];
            while (i + k < n && j + k < n && s[i + k] == s[j + k]) k++;
            lcp[r] = k;
            if (k > 0) k--;
        }
        rmq_n = (i64)lcp.size();
        if (rmq_n > 0) {
            i64 lg = 1;
            while ((1LL << lg) <= rmq_n) lg++;
            rmq.assign(lg * rmq_n, 0);
            for (i64 i = 0; i < rmq_n; i++) rmq[i] = lcp[i];
            for (i64 b = 1; b < lg; b++) {
                i64 step = 1LL << (b - 1);
                i64 lim = rmq_n - (1LL << b) + 1;
                i64 cur = b * rmq_n, prev = (b - 1) * rmq_n;
                for (i64 i = 0; i < lim; i++)
                    rmq[cur + i] = std::min(rmq[prev + i], rmq[prev + i + step]);
            }
        }
    }
    i64 lcp_between(i64 i, i64 j) const {
        if (i == j) return (i64)s.size() - i;
        i64 a = std::min(rank[i], rank[j]), b = std::max(rank[i], rank[j]);
        i64 k = 63 - __builtin_clzll((unsigned long long)(b - a));
        i64 base = k * rmq_n;
        return std::min(rmq[base + a], rmq[base + b - (1LL << k)]);
    }
};

int main() {
    const i64 n = 100000;
    std::vector<i64> s(n);
    i64 x = 246813579;
    for (i64 i = 0; i < n; i++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        s[i] = (x >> 16) & 3;
    }
    SuffixArray sa(s);
    std::vector<i64> rev(s.rbegin(), s.rend());
    SuffixArray rsa(rev);
    std::vector<std::tuple<i64, i64, i64>> raw;
    for (int rev_ord = 0; rev_ord < 2; rev_ord++) {
        std::vector<i64> st;
        for (i64 i = n - 1; i >= 0; i--) {
            while (!st.empty()) {
                i64 top = st.back();
                i64 common = sa.lcp_between(i, top);
                if (top + common == n) break;
                bool less = (s[i + common] < s[top + common]) != (bool)rev_ord;
                if (less) st.pop_back(); else break;
            }
            i64 j = st.empty() ? n : st.back();
            st.push_back(i);
            i64 p = j - i;
            i64 left = (i == 0) ? 0 : rsa.lcp_between(n - i, n - j);
            i64 right = (j == n) ? 0 : sa.lcp_between(i, j);
            i64 l = i - left, r = j + right;
            if (r - l >= 2 * p) raw.emplace_back(l, r, p);
        }
    }
    std::sort(raw.begin(), raw.end());
    std::vector<std::tuple<i64, i64, i64>> out;
    for (auto& run : raw) {
        if (out.empty() || std::get<0>(out.back()) != std::get<0>(run) || std::get<1>(out.back()) != std::get<1>(run))
            out.push_back(run);
    }
    i64 ans = (i64)out.size();
    for (auto& [l, r, p] : out) ans += l * 3 + r * 5 + p * 7;
    std::cout << ans << "\n";
}
