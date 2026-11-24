// Official editorial: reflect the grid over its borders forever. Then types 1-4 swap columns
// (rows) 2k, 2k+1 or 2k+1, 2k+2 everywhere, and type 5 rotates 2 x 2 block (A, B) clockwise
// when A + B is even and counterclockwise otherwise. These moves commute with translation by
// 4, so 16 tracked cells give every cell's displacement; fold back into the grid at the end.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
ll fold(ll p, ll len) {
    ll q = ((p % (2 * len)) + 2 * len) % (2 * len);
    return q < len ? q : 2 * len - 1 - q;
}
ll fdiv(ll a) { return a >= 0 ? a / 2 : -((-a + 1) / 2); }
int main() {
    ll n, m;
    int q;
    scanf("%lld %lld %d", &n, &m, &q);
    vector<int> t(q);
    for (auto& x : t) scanf("%d", &x);
    ll R = 2 * n, C = 2 * m;
    ll dx[4][4], dy[4][4];
    for (int a = 0; a < 4; a++)
        for (int b = 0; b < 4; b++) {
            ll x = a, y = b;
            for (int op : t) {
                if (op == 1) y += (y & 1) ? -1 : 1;
                else if (op == 2) y += (y & 1) ? 1 : -1;
                else if (op == 3) x += (x & 1) ? -1 : 1;
                else if (op == 4) x += (x & 1) ? 1 : -1;
                else {
                    ll bx = fdiv(x), by = fdiv(y);
                    int u = x & 1, v = y & 1;
                    bool cw = ((bx + by) % 2 + 2) % 2 == 0;
                    int nu, nv;
                    if (cw) nu = v, nv = 1 - u;
                    else nu = 1 - v, nv = u;
                    x = 2 * bx + nu, y = 2 * by + nv;
                }
            }
            dx[a][b] = x - a, dy[a][b] = y - b;
        }
    vector<int> g(R * C);
    for (ll i = 0; i < R; i++)
        for (ll j = 0; j < C; j++) {
            ll x = fold(i + dx[i % 4][j % 4], R), y = fold(j + dy[i % 4][j % 4], C);
            g[x * C + y] = i * C + j + 1;
        }
    string out;
    out.reserve(R * C * 8);
    char buf[16];
    for (ll i = 0; i < R; i++)
        for (ll j = 0; j < C; j++) {
            int len = snprintf(buf, sizeof buf, "%d", g[i * C + j]);
            out.append(buf, len);
            out += j + 1 < C ? ' ' : '\n';
        }
    fwrite(out.data(), 1, out.size(), stdout);
}
