// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
int main() {
    string s;
    cin >> s;
    long long cnt[26] = {}, ans = 1, mod = 1000000007;
    for (char c : s) cnt[c - 'a']++;
    for (int i = 0; i < 26; i++) ans = ans * (cnt[i] + 1) % mod;
    cout << ans << '\n';
}
