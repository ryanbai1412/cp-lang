#include <bits/stdc++.h>
using namespace std;
const int MOD = 998244353, N = 5000;
int main() {
    // A set is one-character (cost |M|) or an alternating window of 2 or 3 adjacent positions
    // (cost 2|M|). Max cost = n + total length of maximal alternating runs of length >= 2;
    // count = prod(tilings of each run by 2s and 3s) * Bell(zeros left) * Bell(ones left).
    vector<long long> bell(N + 1), row{1}, nxt;
    bell[0] = 1;
    for (int i = 1; i <= N; i++) {
        nxt.assign(1, row.back());
        for (long long x : row) nxt.push_back((nxt.back() + x) % MOD);
        row.swap(nxt);
        bell[i] = row[0];
    }
    vector<long long> til(N + 1);
    til[0] = 1;
    for (int i = 2; i <= N; i++) til[i] = (til[i - 2] + (i >= 3 ? til[i - 3] : 0)) % MOD;
    int t;
    scanf("%d", &t);
    static char s[N + 2];
    while (t--) {
        int n;
        scanf("%d %s", &n, s);
        long long cost = n, ways = 1;
        int left[2] = {0, 0};
        for (int i = 0; i < n;) {
            int j = i + 1;
            while (j < n && s[j] != s[j - 1]) j++;
            int len = j - i;
            if (len == 1) left[s[i] - '0']++;
            else cost += len, ways = ways * til[len] % MOD;
            i = j;
        }
        ways = ways * bell[left[0]] % MOD * bell[left[1]] % MOD;
        printf("%lld %lld\n", cost, ways);
    }
}
