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
#define sz(x) int((x).size())

void solve() {
	vector<tuple<int, int, int>> edges;
	for (int i = 1; i < 5e4; i++) {
		edges.emplace_back(0, i, 1e8 - i);
		edges.emplace_back(i, 5e4, 1);
	}
	for (int i = 5e4 + 1; i < 1e5; i++) {
		edges.emplace_back(5e4, i, 1);
	}
	int n = 1e5, m = sz(edges);
	cout << n << " " << m << endl;
	for (auto& [u, v, w] : edges) {
		cout << u + 1 << " " << v + 1 << " " << w << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}