#include <bits/stdc++.h>
using namespace std;

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, reps;
    if (!(cin >> n >> reps)) return 0;
    vector<int64_t> a(n);
    for (auto& x : a) cin >> x;
    int64_t acc = 0;
    for (int r = 0; r < reps; ++r) {
        static vector<int64_t> st;
        st.clear();
        for (int64_t x : a) {
            if (!st.empty() && st.back() > x) ++acc;
            if (x % 3 != r % 3) st.push_back(x);
        }
        if (!st.empty()) acc += st.back();
        acc += st.front();
    }
    cout << acc << '\n';
}
