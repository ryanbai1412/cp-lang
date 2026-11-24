// The answer is the sum of |T| + 1 over the distinct substrings T of S[l..r]. Sweeping the
// right end i, A_j holds the contribution of the substrings whose last occurrence in S[1..i]
// starts at j, so a round asks for A_l + ... + A_r after step r (official editorial). The
// suffix link tree of the SAM of S is kept in a link/cut tree whose nodes remember the last
// end position; accessing the prefix state at step i visits each run of equal last ends
// once, and each run subtracts an arithmetic progression from A. Rounds are online (XOR
// with the last answer), so instead of a persistent tree the suffix sums of A are stored
// every B steps and the at most B steps of updates since are added per round.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
const int B = 256;
struct SAM {
    vector<array<int, 26>> nx;
    vector<int> link, len;
    int last = 0;
    SAM(int n) { nx.reserve(2 * n + 2), link.reserve(2 * n + 2), len.reserve(2 * n + 2), add_state(0, -1); }
    int add_state(int l, int lk) {
        nx.push_back({}), nx.back().fill(-1), link.push_back(lk), len.push_back(l);
        return len.size() - 1;
    }
    int extend(int c) {
        int cur = add_state(len[last] + 1, 0), p = last;
        while (p != -1 && nx[p][c] == -1) nx[p][c] = cur, p = link[p];
        if (p != -1) {
            int q = nx[p][c];
            if (len[p] + 1 == len[q]) link[cur] = q;
            else {
                int cl = add_state(len[p] + 1, link[q]);
                nx[cl] = nx[q];
                while (p != -1 && nx[p][c] == q) nx[p][c] = cl, p = link[p];
                link[q] = link[cur] = cl;
            }
        }
        return last = cur;
    }
};
vector<int> ch0, ch1, fa, val, tag, low, mn;
bool isroot(int x) { return fa[x] < 0 || (ch0[fa[x]] != x && ch1[fa[x]] != x); }
void pull(int x) {
    mn[x] = low[x];
    if (ch0[x] >= 0) mn[x] = min(mn[x], mn[ch0[x]]);
    if (ch1[x] >= 0) mn[x] = min(mn[x], mn[ch1[x]]);
}
void apply(int x, int v) {
    if (x >= 0) val[x] = tag[x] = v;
}
void push(int x) {
    if (tag[x]) apply(ch0[x], tag[x]), apply(ch1[x], tag[x]), tag[x] = 0;
}
void rotate(int x) {
    int y = fa[x], z = fa[y];
    bool r = ch1[y] == x;
    if (!isroot(y)) (ch0[z] == y ? ch0[z] : ch1[z]) = x;
    fa[x] = z;
    int b = r ? ch0[x] : ch1[x];
    (r ? ch1[y] : ch0[y]) = b;
    if (b >= 0) fa[b] = y;
    (r ? ch0[x] : ch1[x]) = y, fa[y] = x;
    pull(y), pull(x);
}
void splay(int x) {
    static vector<int> st;
    st.clear();
    for (int y = x;; y = fa[y]) {
        st.push_back(y);
        if (isroot(y)) break;
    }
    for (int k = st.size() - 1; k >= 0; k--) push(st[k]);
    while (!isroot(x)) {
        int y = fa[x];
        if (!isroot(y)) rotate((ch0[y] == x) == (ch0[fa[y]] == y) ? y : x);
        rotate(x);
    }
}
struct Upd { int lo, hi, sign; ll c; };  // A_j += sign * (c - j) for lo <= j <= hi
int main() {
    static char buf[100005];
    scanf("%s", buf);
    int n = strlen(buf);
    SAM sam(n);
    vector<int> pos(n + 1);
    for (int i = 1; i <= n; i++) pos[i] = sam.extend(buf[i - 1] - 'a');
    int m = sam.len.size();
    ch0.assign(m, -1), ch1.assign(m, -1), fa.assign(m, -1), val.assign(m, 0), tag.assign(m, 0), low.assign(m, 1), mn.assign(m, 1);
    for (int v = 1; v < m; v++) fa[v] = sam.link[v], low[v] = mn[v] = sam.len[sam.link[v]] + 1;
    vector<Upd> ups;
    vector<int> stepStart(n + 2);
    vector<ll> dC(n + 2), dJ(n + 2);
    vector<vector<ll>> snap(n / B + 1);
    auto add = [&](int lo, int hi, int sign, ll c) {
        if (lo > hi) return;
        ups.push_back({lo, hi, sign, c});
        dC[lo] += sign * c, dC[hi + 1] -= sign * c, dJ[lo] += sign, dJ[hi + 1] -= sign;
    };
    for (int i = 1; i <= n; i++) {
        stepStart[i] = ups.size();
        for (int x = pos[i], y = -1; x >= 0; y = x, x = fa[x]) {
            splay(x);
            if (val[x]) {
                int p = val[x], a = ch0[x] >= 0 ? mn[ch0[x]] : low[x], b = sam.len[x];
                if (a <= b) add(p - b + 1, p - a + 1, -1, p + 2);
            }
            ch1[x] = y, pull(x);
        }
        splay(pos[i]), apply(pos[i], i);
        add(1, i, 1, i + 2);
        if (i % B == 0) {
            vector<ll>& s = snap[i / B];
            s.assign(i + 2, 0);
            ll C = 0, J = 0;
            vector<ll> a(i + 1);
            for (int j = 1; j <= i; j++) C += dC[j], J += dJ[j], a[j] = C - J * j;
            for (int j = i; j >= 1; j--) s[j] = s[j + 1] + a[j];
        }
    }
    stepStart[n + 1] = ups.size();
    int q;
    scanf("%d", &q);
    ll last = 0;
    string out;
    while (q--) {
        ll lp, rp;
        scanf("%lld %lld", &lp, &rp);
        int l = lp ^ last, r = rp ^ last, k = r / B;
        ll ans = k && l <= k * B ? snap[k][l] : 0;
        for (int u = stepStart[k * B + 1]; u < stepStart[r + 1]; u++) {
            const Upd& e = ups[u];
            if (e.hi < l) continue;
            ll lo = max(e.lo, l), cnt = e.hi - lo + 1;
            ans += e.sign * (e.c * cnt - (lo + e.hi) * cnt / 2);
        }
        out += to_string(ans), out += '\n';
        last = ans;
    }
    fputs(out.c_str(), stdout);
}
