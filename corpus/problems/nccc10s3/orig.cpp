// Reference for nccc10s3: suffix array (prefix doubling with radix sort), then the k-th
// smallest start in a pattern's rank range from a persistent segment tree over the ranks.
// O(n log n) build, O(|t| log n + log n) per query.
#include <bits/stdc++.h>
using namespace std;

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    string s;
    cin >> s;
    int n = s.size();
    vector<int> sa(n), rk(n), tmp(n), cnt(max(n, 256) + 1);
    for (int i = 0; i < n; i++) sa[i] = i, rk[i] = (unsigned char)s[i];
    sort(sa.begin(), sa.end(), [&](int a, int b) { return rk[a] < rk[b]; });
    for (int len = 1;; len <<= 1) {
        auto key = [&](int i) { return i + len < n ? rk[i + len] + 1 : 0; };
        int m = max(n, 256) + 1;
        vector<int> by(n);
        fill(cnt.begin(), cnt.end(), 0);
        for (int i = 0; i < n; i++) cnt[key(i)]++;
        for (int i = 1; i < m; i++) cnt[i] += cnt[i - 1];
        for (int i = n - 1; i >= 0; i--) by[--cnt[key(i)]] = i;
        fill(cnt.begin(), cnt.end(), 0);
        for (int i = 0; i < n; i++) cnt[rk[i]]++;
        for (int i = 1; i < m; i++) cnt[i] += cnt[i - 1];
        for (int i = n - 1; i >= 0; i--) sa[--cnt[rk[by[i]]]] = by[i];
        tmp[sa[0]] = 0;
        for (int i = 1; i < n; i++)
            tmp[sa[i]] = tmp[sa[i - 1]] + (rk[sa[i]] != rk[sa[i - 1]] || key(sa[i]) != key(sa[i - 1]));
        rk = tmp;
        if (rk[sa[n - 1]] == n - 1) break;
    }
    // root[i] counts the starts sa[0..i)
    vector<int> lc(1), rc(1), sum(1), root(n + 1);
    auto add = [&](auto& self, int prev, int lo, int hi, int at) -> int {
        int cur = lc.size();
        lc.push_back(lc[prev]), rc.push_back(rc[prev]), sum.push_back(sum[prev] + 1);
        if (hi - lo > 1) {
            int mid = (lo + hi) / 2;
            if (at < mid) { int c = self(self, lc[prev], lo, mid, at); lc[cur] = c; }
            else { int c = self(self, rc[prev], mid, hi, at); rc[cur] = c; }
        }
        return cur;
    };
    for (int i = 0; i < n; i++) root[i + 1] = add(add, root[i], 0, n, sa[i]);
    auto cmp = [&](int start, const string& t) {  // sign of suffix start's first |t| chars vs t
        int lim = min(n - start, (int)t.size());
        for (int k = 0; k < lim; k++)
            if (s[start + k] != t[k]) return s[start + k] < t[k] ? -1 : 1;
        return n - start < (int)t.size() ? -1 : 0;
    };
    int q;
    cin >> q;
    string out;
    while (q--) {
        string t;
        long long k;
        cin >> t >> k;
        int lo = 0, hi = n;
        while (lo < hi) { int mid = (lo + hi) / 2; if (cmp(sa[mid], t) < 0) lo = mid + 1; else hi = mid; }
        int l = lo;
        hi = n;
        while (lo < hi) { int mid = (lo + hi) / 2; if (cmp(sa[mid], t) <= 0) lo = mid + 1; else hi = mid; }
        int r = lo;
        if (k > r - l) { out += "-1\n"; continue; }
        int a = root[l], b = root[r], left = 0, right = n;
        long long need = k;
        while (right - left > 1) {
            int mid = (left + right) / 2, here = sum[lc[b]] - sum[lc[a]];
            if (need <= here) a = lc[a], b = lc[b], right = mid;
            else need -= here, a = rc[a], b = rc[b], left = mid;
        }
        out += to_string(left + 1) + "\n";
    }
    cout << out;
}
