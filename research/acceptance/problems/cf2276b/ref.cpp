#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int t; cin >> t;
    string s;
    while (t--) {
        cin >> s;
        bool grow = false, shrink = false;
        for (size_t i = 1; i + 1 < s.size(); i++) {
            if (s[i - 1] == s[i] && s[i] == s[i + 1]) grow = true;
            if (s[i - 1] == s[i + 1] && s[i] != s[i - 1]) shrink = true;
        }
        cout << 1 + grow + shrink << '\n';
    }
}
