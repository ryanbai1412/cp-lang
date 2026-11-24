// yac1p5 reference. Kangaroo i is at s + v t + a t(t-1)/2 at time t. If every kangaroo in
// [l, r] meets at some t >= 1, moving one of them by an offset outside a finite (or one-sided
// bounded) bad set breaks every such meeting, so each answer is 0 or 1. All of [l, r] meet at t
// exactly when t is a common root of every adjacent pair's difference; a pair's difference
// is a nonzero quadratic (distinct starts), so it has at most two positive integer roots.
// A segment tree over adjacent pairs keeps the intersection of those root sets.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
typedef __int128 L;

int n, q;
vector<ll> S, Vv, Aa;
struct Roots { int k = 0; ll t[2]; };

Roots pairRoots(int i, int j) {
    ll A = Aa[j] - Aa[i], B = 2 * (Vv[j] - Vv[i]) - A, C = 2 * (S[j] - S[i]);
    Roots r;
    auto add = [&](ll t) {
        if (t < 1) return;
        if ((L)A * t * t + (L)B * t + C != 0) return;
        for (int k = 0; k < r.k; k++) if (r.t[k] == t) return;
        r.t[r.k++] = t;
    };
    if (A == 0) {
        if (B != 0 && C % B == 0) add(-C / B);
    } else {
        L d = (L)B * B - (L)4 * A * C;
        if (d >= 0) {
            ll sq = (ll)sqrtl((long double)d);
            while ((L)sq * sq > d) sq--;
            while ((L)(sq + 1) * (sq + 1) <= d) sq++;
            if ((L)sq * sq == d)
                for (ll num : {-B + sq, -B - sq})
                    if (num % (2 * A) == 0) add(num / (2 * A));
        }
    }
    if (r.k == 2 && r.t[0] > r.t[1]) swap(r.t[0], r.t[1]);
    return r;
}

Roots meet(const Roots& x, const Roots& y) {
    Roots r;
    for (int a = 0; a < x.k; a++)
        for (int b = 0; b < y.k; b++)
            if (x.t[a] == y.t[b]) r.t[r.k++] = x.t[a];
    return r;
}

int sz;
vector<Roots> tr;
void setPair(int p) {  // pair p joins kangaroos p and p + 1 (0-based)
    int o = p + sz;
    tr[o] = pairRoots(p, p + 1);
    for (o /= 2; o; o /= 2) tr[o] = meet(tr[2 * o], tr[2 * o + 1]);
}
bool anyMeet(int lo, int hi) {  // pairs lo..hi inclusive
    Roots left, right;
    bool hasL = false, hasR = false;
    for (int a = lo + sz, b = hi + sz + 1; a < b; a /= 2, b /= 2) {
        if (a & 1) { left = hasL ? meet(left, tr[a]) : tr[a]; hasL = true; a++; }
        if (b & 1) { --b; right = hasR ? meet(tr[b], right) : tr[b]; hasR = true; }
    }
    Roots r = hasL && hasR ? meet(left, right) : hasL ? left : right;
    return r.k > 0;
}

int main() {
    scanf("%d %d", &n, &q);
    S.resize(n); Vv.resize(n); Aa.resize(n);
    for (int i = 0; i < n; i++) scanf("%lld %lld %lld", &S[i], &Vv[i], &Aa[i]);
    sz = 1;
    while (sz < n) sz *= 2;
    tr.assign(2 * sz, Roots());
    for (int p = 0; p + 1 < n; p++) tr[p + sz] = pairRoots(p, p + 1);
    for (int o = sz - 1; o; o--) tr[o] = meet(tr[2 * o], tr[2 * o + 1]);
    string out;
    while (q--) {
        int t;
        scanf("%d", &t);
        if (t == 1) {
            int i;
            ll x, y, z;
            scanf("%d %lld %lld %lld", &i, &x, &y, &z);
            i--;
            S[i] = x, Vv[i] = y, Aa[i] = z;
            if (i > 0) setPair(i - 1);
            if (i + 1 < n) setPair(i);
        } else {
            int l, r;
            scanf("%d %d", &l, &r);
            out += anyMeet(l - 1, r - 2) ? "1\n" : "0\n";
        }
    }
    fputs(out.c_str(), stdout);
}
