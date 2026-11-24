// Not a judge-accepted submission: written for the corpus and checked against the samples and an exhaustive brute.
#include <bits/stdc++.h>
using namespace std;
using Rules = vector<pair<string, string>>;
// Every string within 5 rewrites of `from`, with its fewest rewrites (breadth first, each string once).
unordered_map<string, int> reach(const string& from, const Rules& rules) {
    unordered_map<string, int> dist{{from, 0}};
    vector<string> layer{from};
    for (int d = 1; d <= 5 && !layer.empty(); d++) {
        vector<string> next;
        for (auto& s : layer)
            for (auto& [a, b] : rules)
                for (size_t i = s.find(a); i != string::npos; i = s.find(a, i + 1)) {
                    string t = s.substr(0, i) + b + s.substr(i + a.size());
                    if (dist.emplace(t, d).second) next.push_back(move(t));
                }
        layer = move(next);
    }
    return dist;
}
int main() {
    string A, B, a, b;
    cin >> A >> B;
    Rules fwd, back;
    while (cin >> a >> b) fwd.emplace_back(a, b), back.emplace_back(b, a);
    auto da = reach(A, fwd), db = reach(B, back);
    int best = INT_MAX;
    for (auto& [s, d] : da)
        if (auto it = db.find(s); it != db.end()) best = min(best, d + it->second);
    if (best > 10) puts("NO ANSWER!");
    else printf("%d\n", best);
}
