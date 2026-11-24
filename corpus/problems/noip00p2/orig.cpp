// Not a judge-accepted submission: written for the corpus and checked against the samples and an exhaustive brute.
#include <bits/stdc++.h>
using namespace std;
using Big = vector<uint32_t>;  // base 1e9, little endian, no leading zero limbs; zero is {}
const uint64_t BASE = 1000000000;
Big fromDigits(const string& s) {
    Big r;
    for (int e = (int)s.size(); e > 0; e -= 9) {
        int b = max(0, e - 9);
        r.push_back(stoul(s.substr(b, e - b)));
    }
    while (!r.empty() && r.back() == 0) r.pop_back();
    return r;
}
Big mul(const Big& a, const Big& b) {
    if (a.empty() || b.empty()) return {};
    vector<uint64_t> t(a.size() + b.size() + 1);
    for (size_t i = 0; i < a.size(); i++) {
        uint64_t c = 0;
        for (size_t j = 0; j < b.size() || c; j++) {
            uint64_t cur = t[i + j] + c + (j < b.size() ? (uint64_t)a[i] * b[j] : 0);
            t[i + j] = cur % BASE, c = cur / BASE;
        }
    }
    Big r(t.begin(), t.end());
    while (!r.empty() && r.back() == 0) r.pop_back();
    return r;
}
bool less_(const Big& a, const Big& b) {
    if (a.size() != b.size()) return a.size() < b.size();
    for (size_t i = a.size(); i-- > 0;)
        if (a[i] != b[i]) return a[i] < b[i];
    return false;
}
int main() {
    int n, k;
    string s;
    cin >> n >> k >> s;
    // best[i][j]: the largest product of s[0..i) cut into j + 1 numbers
    vector<vector<optional<Big>>> best(n + 1, vector<optional<Big>>(k + 1));
    for (int i = 1; i <= n; i++) best[i][0] = fromDigits(s.substr(0, i));
    for (int j = 1; j <= k; j++)
        for (int i = j + 1; i <= n; i++)
            for (int c = j; c < i; c++)
                if (best[c][j - 1]) {
                    Big p = mul(*best[c][j - 1], fromDigits(s.substr(c, i - c)));
                    if (!best[i][j] || less_(*best[i][j], p)) best[i][j] = p;
                }
    const Big& r = *best[n][k];
    if (r.empty()) { puts("0"); return 0; }
    printf("%u", r.back());
    for (size_t i = r.size() - 1; i-- > 0;) printf("%09u", r[i]);
    puts("");
}
