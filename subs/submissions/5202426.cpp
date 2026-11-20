#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

int Connected(int a, int i, int j);
void DescribeDesign(std::vector<std::pair<int,int>> result);

void ToyDesign(int n, int max_ops) {
    vector<pair<int, int>> ans;
    for (int i = 1; i <= n; i++) {
        for (int j = 1; j < i; j++) {
            if (!Connected(0, i, j)) {
                ans.emplace_back(i, j);
            }
        }
    }
    DescribeDesign(ans);
}

void solve() {

}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}