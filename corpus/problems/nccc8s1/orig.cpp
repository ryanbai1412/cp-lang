// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
int main() {
    string al, s;
    cin >> al >> s;
    int n = s.size();
    set<string> out;
    for (int i = 0; i <= n; i++) {
        if (i < n) out.insert(s.substr(0, i) + s.substr(i + 1));
        for (char c : al) {
            if (i < n) out.insert(s.substr(0, i) + c + s.substr(i + 1));
            out.insert(s.substr(0, i) + c + s.substr(i));
        }
    }
    out.erase(s);
    for (auto& w : out) cout << w << '\n';
}
