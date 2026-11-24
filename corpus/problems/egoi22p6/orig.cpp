// Queen or rook: 1 if any piece reaches in one move, else 2. Bishop (no Q/R): 1 or 2 on the
// same colour, and across colours 2 if some knight/king/pawn step pairs with a bishop or step
// move, else 3 (or -1 with the bishop alone). Without sliders: the king alone is Chebyshev
// distance and the pawn adds nothing to it; the pawn alone goes straight up. Two pawns equal
// two knight moves ((1, 2) + (1, -2)), so knight + pawn uses at most one pawn; knight + king
// uses only a few king moves, searched over all king offsets with |u| <= 6. Knight distance
// is the closed form max(ceil(x/2), ceil((x+y)/3)) fixed to the parity of x + y, with the
// exceptions (1, 0) and (2, 2).
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
ll knight(ll x, ll y) {
    x = llabs(x), y = llabs(y);
    if (x < y) swap(x, y);
    if (x == 1 && y == 0) return 3;
    if (x == 2 && y == 2) return 4;
    ll d = max((x + 1) / 2, (x + y + 2) / 3);
    if ((d + x + y) % 2) d++;
    return d;
}
int main() {
    int q;
    scanf("%d", &q);
    const int KN[8][2] = {{1, 2}, {1, -2}, {-1, 2}, {-1, -2}, {2, 1}, {2, -1}, {-2, 1}, {-2, -1}};
    while (q--) {
        char s[16];
        ll a, b, c, d;
        scanf("%s %lld %lld %lld %lld", s, &a, &b, &c, &d);
        string p = s;
        bool Q = p.find('Q') != string::npos, R = p.find('R') != string::npos, B = p.find('B') != string::npos;
        bool N = p.find('N') != string::npos, K = p.find('K') != string::npos, P = p.find('P') != string::npos;
        if (Q) R = B = true;
        ll dr = c - a, dc = d - b;
        auto stepOK = [&](ll x, ll y) {
            if (x == 0 && y == 0) return false;
            if (N && ((llabs(x) == 1 && llabs(y) == 2) || (llabs(x) == 2 && llabs(y) == 1))) return true;
            if (K && llabs(x) <= 1 && llabs(y) <= 1) return true;
            if (P && x == 1 && y == 0) return true;
            return false;
        };
        auto oneOK = [&](ll x, ll y) {
            if (x == 0 && y == 0) return false;
            if (R && (x == 0 || y == 0)) return true;
            if (B && llabs(x) == llabs(y)) return true;
            return stepOK(x, y);
        };
        ll ans;
        if (oneOK(dr, dc)) ans = 1;
        else if (R) ans = 2;
        else if (B) {
            if ((dr + dc) % 2 == 0) ans = 2;
            else if (!N && !K && !P) ans = -1;
            else {
                vector<pair<ll, ll>> steps;
                if (N)
                    for (auto& m : KN) steps.push_back({m[0], m[1]});
                if (K)
                    for (int x = -1; x <= 1; x++)
                        for (int y = -1; y <= 1; y++)
                            if (x || y) steps.push_back({x, y});
                if (P) steps.push_back({1, 0});
                bool two = false;
                for (auto [x, y] : steps) two = two || oneOK(dr - x, dc - y);
                ans = two ? 2 : 3;
            }
        } else if (N) {
            if (K) {
                ans = LLONG_MAX;
                for (int x = -6; x <= 6; x++)
                    for (int y = -6; y <= 6; y++) ans = min(ans, (ll)max(abs(x), abs(y)) + knight(dr - x, dc - y));
            } else if (P) ans = min(knight(dr, dc), 1 + knight(dr - 1, dc));
            else ans = knight(dr, dc);
        } else if (K) ans = max(llabs(dr), llabs(dc));
        else ans = dc == 0 && dr > 0 ? dr : -1;
        printf("%lld\n", ans);
    }
}
