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
	const int n = 1e5, mid = n / 2;
	vector<tuple<int, int, int>> edges;
	for (int i = 1; i < mid; i++) {
		edges.emplace_back(0, i, i);
		edges.emplace_back(i, mid, 1e8 - 1000 * i);
	}
	for (int i = mid + 1; i < n; i++) {
		edges.emplace_back(mid, i, 1);
	}
	int m = sz(edges);
	assert(m <= 2e5);
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