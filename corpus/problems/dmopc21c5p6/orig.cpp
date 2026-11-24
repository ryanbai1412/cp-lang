// Official editorial: greedily give p_i the smallest free value with i + p_i prime (p_1 = 2
// when N is even, by parity), then find the lexicographically smallest perfect matching on
// the last F positions by rerouting alternating paths. If the greedy prefix leaves no perfect
// matching, F doubles (up to N, which is the plain lexicographic matching).
#include <bits/stdc++.h>
using namespace std;
int n;
vector<char> comp;
bool prime(int x) { return x >= 2 && !comp[x]; }
vector<int> L, R, matchL, matchR, seen;
vector<char> fixedL, fixedR;
vector<vector<int>> adj;
int stamp = 0;
bool aug(int u) {
    for (int v : adj[u]) {
        if (fixedR[v] || seen[v] == stamp) continue;
        seen[v] = stamp;
        if (matchR[v] < 0 || aug(matchR[v])) {
            matchL[u] = v, matchR[v] = u;
            return true;
        }
    }
    return false;
}
// Lexicographically smallest perfect matching of positions L to values R; false if none.
bool lexmatch(vector<int>& p) {
    int f = L.size();
    adj.assign(f, {});
    for (int a = 0; a < f; a++)
        for (int b = 0; b < f; b++)
            if (prime(L[a] + R[b])) adj[a].push_back(b);
    matchL.assign(f, -1), matchR.assign(f, -1), seen.assign(f, 0);
    fixedL.assign(f, 0), fixedR.assign(f, 0);
    for (int a = 0; a < f; a++) {
        ++stamp;
        if (!aug(a)) return false;
    }
    for (int a = 0; a < f; a++) {
        for (int b : adj[a]) {
            if (fixedR[b]) continue;
            if (matchL[a] == b) break;
            int a2 = matchR[b], b0 = matchL[a];
            fixedL[a] = 1, fixedR[b] = 1;
            matchL[a2] = -1, matchR[b0] = -1;
            ++stamp;
            if (aug(a2)) {
                matchL[a] = b, matchR[b] = a;
                break;
            }
            fixedR[b] = 0;
            matchL[a2] = b, matchR[b0] = a;
        }
        fixedL[a] = 1, fixedR[matchL[a]] = 1;
    }
    for (int a = 0; a < f; a++) p[L[a]] = R[matchL[a]];
    return true;
}
int main() {
    scanf("%d", &n);
    comp.assign(2 * n + 2, 0);
    for (int i = 2; (long long)i * i <= 2 * n + 1; i++)
        if (!comp[i])
            for (int j = i * i; j <= 2 * n + 1; j += i) comp[j] = 1;
    vector<int> p(n + 1);
    for (int F = min(n, 400);; F = min(n, 2 * F)) {
        int pre = n - F;
        set<int> freev;
        for (int v = 1; v <= n; v++) freev.insert(v);
        bool ok = true;
        for (int i = 1; i <= pre && ok; i++) {
            auto it = freev.begin();
            if (i == 1 && n % 2 == 0) it = freev.find(2);
            while (it != freev.end() && !prime(i + *it)) ++it;
            if (it == freev.end()) ok = false;
            else p[i] = *it, freev.erase(it);
        }
        if (ok) {
            L.clear(), R.assign(freev.begin(), freev.end());
            for (int i = pre + 1; i <= n; i++) L.push_back(i);
            if (lexmatch(p)) break;
        }
        if (F == n) {
            puts("-1");
            return 0;
        }
    }
    string out;
    for (int i = 1; i <= n; i++) out += to_string(p[i]), out += i < n ? ' ' : '\n';
    fputs(out.c_str(), stdout);
}
