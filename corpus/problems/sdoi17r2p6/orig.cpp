// sdoi17r2p6 (SDOI 2017 Text Correction) reference. T = A B C and S is one of the six
// orders of the three parts; each order is tested in O(n) or O(n log n):
//  ABC: S == T.  BCA, CAB: S is a rotation of T (KMP of S in TT).
//  ACB: A <= lcp(S, T); for c = |C|, b = |B|, S[a, a+c) is a suffix of T (a Z-array on the
//    reversed strings) and T[a, a+b) is a suffix of S (the KMP failure chain of reversed S
//    over reversed T), with the chain's minimum of the first bound kept per node.
//  BAC: ACB on the reversed strings.
//  CBA: interleave R = S0 T(n-1) S1 T(n-2) ...; then S = CBA exactly when R splits into
//    three even palindromes at even offsets of lengths 2c, 2b, 2a. For each valid first
//    piece the rest splits into two such palindromes iff it does with the longest proper
//    one as the last piece or the longest proper one as the middle piece; Manacher plus a
//    segment tree descent finds both.
#include <bits/stdc++.h>
using namespace std;

static char buf[1 << 25];
int bl = 0, bp = 0;
inline int gc() {
    if (bp == bl) { bl = fread(buf, 1, sizeof buf, stdin); bp = 0; if (bl <= 0) return -1; }
    return buf[bp++];
}
inline int rd() {
    int c = gc();
    while (c != '-' && (c < '0' || c > '9')) c = gc();
    int x = 0;
    while (c >= '0' && c <= '9') { x = x * 10 + c - '0'; c = gc(); }
    return x;
}

vector<int> zfun(const vector<int>& s) {
    int n = s.size();
    vector<int> z(n, 0);
    if (n) z[0] = n;
    for (int i = 1, l = 0, r = 0; i < n; i++) {
        if (i < r) z[i] = min(r - i, z[i - l]);
        while (i + z[i] < n && s[z[i]] == s[i + z[i]]) z[i]++;
        if (i + z[i] > r) l = i, r = i + z[i];
    }
    return z;
}
vector<int> pref(const vector<int>& s) {
    int n = s.size();
    vector<int> p(n, 0);
    for (int i = 1; i < n; i++) {
        int k = p[i - 1];
        while (k && s[i] != s[k]) k = p[k - 1];
        if (s[i] == s[k]) k++;
        p[i] = k;
    }
    return p;
}

struct Seg { int l1, r1, l2, r2, l3, r3; };

// T = A B C, S = A C B with |A| = a; returns {a, b, c} or a = 0.
array<int, 3> acb(const vector<int>& S, const vector<int>& T) {
    int n = S.size();
    int P = 0;
    while (P < n && S[P] == T[P]) P++;
    vector<int> rS(S.rbegin(), S.rend()), rT(T.rbegin(), T.rend());
    // Zr[e]: longest l with S[e-l, e) == T[n-l, n) = lcp(rS from n-e, rT).
    vector<int> cat(rT);
    cat.push_back(-1);
    cat.insert(cat.end(), rS.begin(), rS.end());
    vector<int> z = zfun(cat);
    auto Zr = [&](int e) { return e == 0 ? 0 : min(e, z[n + 1 + (n - e)]); };
    vector<int> fail = pref(rS);  // fail of prefix length k is fail[k-1]
    const long long INF = LLONG_MAX / 4;
    vector<long long> mh(n + 1, INF);
    vector<int> arg(n + 1, 0);
    for (int b = 1; b <= n; b++) {
        long long h = (b < n) ? (long long)n - b - Zr(n - b) : INF;
        int f = fail[b - 1];
        mh[b] = h, arg[b] = b;
        if (mh[f] < mh[b]) mh[b] = mh[f], arg[b] = arg[f];
    }
    // KMP of rS over rT: state after rT[0, j)
    vector<int> st(n + 1, 0);
    int k = 0;
    for (int j = 0; j < n; j++) {
        while (k && (k == n || rT[j] != rS[k])) k = fail[k - 1];
        if (rT[j] == rS[k]) k++;
        st[j + 1] = k;
    }
    for (int a = 1; a <= min(P, n - 2); a++) {
        int j = n - a, kk = st[j];
        if (kk == n - a) kk = fail[kk - 1];
        if (kk && mh[kk] <= a) {
            int b = arg[kk], c = n - a - b;
            if (b >= 1 && c >= 1) return {a, b, c};
        }
    }
    return {0, 0, 0};
}

bool cba(const vector<int>& S, const vector<int>& T, Seg& out) {
    int n = S.size(), m = 2 * n;
    vector<int> R(m);
    for (int i = 0; i < n; i++) R[2 * i] = S[i], R[2 * i + 1] = T[n - 1 - i];
    // rad[j]: largest r with R[j-1-k] == R[j+k] for k < r (centre between j-1 and j)
    vector<int> rad(m + 1, 0);
    for (int j = 1, l = 0, r = 0; j < m; j++) {  // even Manacher, window [l, r)
        int k = (j < r) ? min(r - j, rad[l + r - j]) : 0;
        while (j - 1 - k >= 0 && j + k < m && R[j - 1 - k] == R[j + k]) k++;
        rad[j] = k;
        if (j + k > r) l = j - k, r = j + k;
    }
    auto pal = [&](int start, int L) { return L >= 1 && rad[start + L] >= L; };  // R[start, start+2L)
    vector<char> inA(n + 1, 0);
    for (int a = 1; a <= n; a++) inA[a] = pal(2 * (n - a), a);
    vector<int> maxA(n + 1, 0);
    for (int a = 1; a <= n; a++) maxA[a] = inA[a] ? a : maxA[a - 1];
    int sz = 1;
    while (sz < m + 1) sz *= 2;
    vector<int> mn(2 * sz, INT_MAX);
    for (int j = 1; j < m; j++) mn[sz + j] = j - rad[j];
    for (int o = sz - 1; o; o--) mn[o] = min(mn[2 * o], mn[2 * o + 1]);
    function<int(int, int, int, int, int, int)> right = [&](int o, int l, int r, int lo, int hi, int x) -> int {
        if (r < lo || hi < l || mn[o] > x) return -1;
        if (l == r) return l;
        int mid = (l + r) / 2;
        int v = right(2 * o + 1, mid + 1, r, lo, hi, x);
        return v >= 0 ? v : right(2 * o, l, mid, lo, hi, x);
    };
    for (int c = 1; c <= n - 2; c++) {
        if (!pal(0, c)) continue;
        int a = maxA[n - c - 1];
        if (a >= 1 && pal(2 * c, n - a - c)) {
            out = {n - c + 1, n, a + 1, n - c, 1, a};
            return true;
        }
        int j = right(1, 0, sz - 1, 2 * c + 1, n + c - 1, 2 * c);
        if (j >= 0) {
            int L = j - 2 * c;
            a = n - c - L;
            if (a >= 1 && inA[a]) {
                out = {n - c + 1, n, a + 1, n - c, 1, a};
                return true;
            }
        }
    }
    return false;
}

bool solve(const vector<int>& S, const vector<int>& T, Seg& o) {
    int n = S.size();
    if (S == T) { o = {1, 1, 2, 2, 3, n}; return true; }
    {  // rotation: S == T[i, n) + T[0, i)
        vector<int> cat(S);
        cat.push_back(-1);
        cat.insert(cat.end(), T.begin(), T.end());
        cat.insert(cat.end(), T.begin(), T.end());
        vector<int> p = pref(cat);
        for (int i = 1; i < n; i++)
            if (p[n + 1 + i + n - 1] == n) {
                if (n - i >= 2) o = {i + 1, i + 1, i + 2, n, 1, i};
                else o = {i + 1, n, 1, 1, 2, i};
                return true;
            }
    }
    auto r = acb(S, T);
    if (r[0]) {
        int a = r[0], b = r[1], c = r[2];
        o = {1, a, n - c + 1, n, a + 1, a + b};
        return true;
    }
    vector<int> rS(S.rbegin(), S.rend()), rT(T.rbegin(), T.rend());
    r = acb(rS, rT);
    if (r[0]) {
        int a3 = r[0], b3 = r[1], c3 = r[2];
        o = {c3 + 1, c3 + b3, 1, c3, c3 + b3 + 1, n};
        return true;
    }
    return cba(S, T, o);
}

int main() {
    int cases = rd();
    string out;
    while (cases--) {
        int n = rd();
        rd();
        vector<int> S(n), T(n);
        for (auto& x : S) x = rd();
        for (auto& x : T) x = rd();
        Seg o;
        if (solve(S, T, o)) {
            char line[96];
            snprintf(line, sizeof line, "YES\n%d %d\n%d %d\n%d %d\n", o.l1, o.r1, o.l2, o.r2, o.l3, o.r3);
            out += line;
        } else {
            out += "NO\n";
        }
    }
    fputs(out.c_str(), stdout);
}
