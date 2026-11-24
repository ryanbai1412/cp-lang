// Primeness is at most the total variation sum |p_i - p_{i+1}|, whose maximum is
// T = floor(N^2 / 2) - 1. Writing it as sum v * w_v (w = +-2 for peaks and valleys, 0 for
// monotone passes, +-1 at the ends), T forces sorted weights: an alternating path with the
// top half as peaks and the bottom half as valleys, the two middle values at the ends. Its
// edges other than the two end edges differ by at least 3, so an all-prime path alternates
// parity and every peak shares one parity, impossible for N >= 6. So the optimum is at
// most T - 1 there, and T - 1 is reached by swapping the weights of two middle values:
// for N = 2k (ends k - 1 and k + 1, peaks k+2..2k, valleys 1..k-2 and k), or for
// N = 2k + 1 with k odd by making k + 1 a monotone step (ends k and k + 2, peaks
// k+3..2k+1, valleys 1..k-1), with k even (ends k + 1 and k + 3, peaks k + 2 and
// k+4..2k+1, valleys 1..k). Any such path with prime steps has primeness exactly T - 1.
// Parity forces its steps of 2, which split it into two parity classes; each is a
// Hamiltonian path between fixed ends in a bipartite prime-difference graph, found by Posa
// rotation-extension. The program checks the total and aborts if it ever misses.
// N <= 21, where T - 1 can fail (N = 4, 12, 13, 14), comes from an exact bitmask DP.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
const char* SMALL[] = {
    "1",
    "1 2",
    "1 3 2",
    "2 4 1 3",
    "1 4 2 5 3",
    "2 5 3 6 1 4",
    "3 6 1 4 7 2 5",
    "3 8 1 6 4 7 2 5",
    "3 8 1 6 4 9 2 7 5",
    "4 9 2 7 5 10 3 8 1 6",
    "5 10 3 8 1 6 11 4 9 2 7",
    "5 10 3 8 1 12 7 2 9 4 11 6",
    "5 10 3 8 1 12 7 2 13 6 11 4 9",
    "6 11 4 9 2 13 8 1 12 5 10 3 14 7",
    "7 10 3 14 1 12 5 8 11 4 15 2 13 6 9",
    "7 12 1 14 3 16 5 10 8 11 4 15 2 13 6 9",
    "7 12 1 14 3 16 5 10 8 13 2 15 4 17 6 11 9",
    "8 13 2 15 4 17 6 11 9 12 1 14 3 16 5 18 7 10",
    "9 12 1 14 3 16 5 18 7 10 13 2 15 4 17 6 19 8 11",
    "9 14 1 18 5 16 3 20 7 12 10 13 2 15 4 17 6 19 8 11",
    "9 14 1 18 5 16 3 20 7 12 10 15 2 19 6 17 4 21 8 13 11"};
vector<char> isp;  // side vectors below hold 0 or 1
vector<int> pr;
mt19937 rng(1);
// Hamiltonian path from s to t over vs (which holds s and t); side[v] splits the graph
vector<int> attempt(const vector<int>& vs, vector<char>& side, int s, int t, ll cap) {
    int n = isp.size() - 1, P = pr.size();
    vector<int> un[2], at(n + 1), pos(n + 1, -2);
    for (int v : vs) pos[v] = -1;
    for (int v : vs)
        if (v != s && v != t) at[v] = un[(int)side[v]].size(), un[(int)side[v]].push_back(v);
    auto take = [&](int v) {
        auto& u = un[(int)side[v]];
        at[u.back()] = at[v], u[at[v]] = u.back(), u.pop_back();
    };
    auto adj = [&](int u, int w) { return w >= 1 && w <= n && w != t && pos[w] > -2 && side[w] != side[u]; };
    vector<int> path{s};
    pos[s] = 0;
    int m = vs.size();
    for (long long it = 0;; it++) {
        if (it > cap) return {};
        int u = path.back();
        if ((int)path.size() == m - 1 && isp[abs(u - t)] && side[u] != side[t]) break;
        int found = -1;
        auto& opp = un[!side[u]];
        if ((int)opp.size() < P) {
            for (int w : opp)
                if (isp[abs(u - w)]) { found = w; break; }
        } else {
            int r = rng() % P;
            for (int c = 0; c < P && found < 0; c++) {
                int p = pr[(r + c) % P];
                for (int w : {u + p, u - p})
                    if (adj(u, w) && pos[w] == -1) { found = w; break; }
            }
        }
        if (found >= 0) {
            take(found), pos[found] = path.size(), path.push_back(found);
            continue;
        }
        int w = -1;
        for (int tries = 0; tries < 100000 && w < 0; tries++) {
            int p = pr[rng() % P], c = rng() & 1 ? u + p : u - p;
            if (adj(u, c) && pos[c] >= 0 && pos[c] != (int)path.size() - 2) w = c;
        }
        if (w < 0) return {};
        reverse(path.begin() + pos[w] + 1, path.end());
        for (int j = pos[w] + 1; j < (int)path.size(); j++) pos[path[j]] = j;
    }
    path.push_back(t);
    return path;
}
vector<int> hampath(const vector<int>& vs, vector<char>& side, int s, int t) {
    // restart with a doubled step budget, so small cases escape bad starts quickly
    for (int i = 0; i < 40; i++) {
        vector<int> p = attempt(vs, side, s, t, (5LL * vs.size() + 20000) << min(i, 6));
        if (!p.empty()) return p;
    }
    abort();
}
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    if (n <= 21) {
        puts(SMALL[n - 1]);
        return 0;
    }
    isp.assign(n + 1, 1), isp[0] = isp[1] = 0;
    for (int i = 2; (ll)i * i <= n; i++)
        if (isp[i])
            for (int j = i * i; j <= n; j += i) isp[j] = 0;
    for (int i = 2; i <= n; i++)
        if (isp[i]) pr.push_back(i);
    rng.seed(n);
    int k = n / 2;
    vector<char> side(n + 1);
    for (int v = 1; v <= n; v++) side[v] = v > k;  // valley 0, peak 1
    // Hamiltonian paths a0 -> a1 and b0 -> b1: a holds the valleys in [vlo, vhi] of a0's
    // parity and the peaks in [plo, phi] of the other, b the rest; the single step of 2
    // that the parity count forces is the a1 -> b0 junction (or a1 == b0 is shared)
    auto chains = [&](int a0, int a1, int b0, int b1, int vlo, int vhi, int plo, int phi) {
        vector<int> a{a0, a1}, b{b0, b1};
        for (int v = vlo; v <= vhi; v++) (v % 2 == a0 % 2 ? a : b).push_back(v);
        for (int v = plo; v <= phi; v++) (v % 2 != a0 % 2 ? a : b).push_back(v);
        side[a1] = 1;
        vector<int> path = hampath(a, side, a0, a1);
        side[b0] = 0;
        vector<int> rest = hampath(b, side, b0, b1);
        path.insert(path.end(), rest.begin() + (a1 == b0), rest.end());
        return path;
    };
    vector<int> path;
    if (n % 2 == 0) path = chains(k - 1, k + 2, k, k + 1, 1, k - 2, k + 3, n);
    else if (k % 2 == 0) {
        path = {k + 1};
        vector<int> rest = chains(k - 1, k + 2, k, k + 3, 1, k - 2, k + 4, n);
        path.insert(path.end(), rest.begin(), rest.end());
    } else
        path = chains(k, k + 1, k + 1, k + 2, 1, k - 1, k + 3, n);
    ll tot = 0;
    for (int i = 0; i + 1 < n; i++)
        if (isp[abs(path[i] - path[i + 1])]) tot += abs(path[i] - path[i + 1]);
    if ((int)path.size() != n || tot != (ll)n * n / 2 - 2) abort();
    string out;
    for (int v : path) out += to_string(v), out += ' ';
    out.back() = '\n';
    fputs(out.c_str(), stdout);
}
