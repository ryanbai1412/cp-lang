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
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

void solve() {
	int n;
	cin >> n;
	string same[n][2];
	for (auto& [u, v] : same) {
		cin >> u >> v;
	}
	int m;
	cin >> m;
	string diff[m][2];
	for (auto& [u, v] : diff) {
		cin >> u >> v;
	}
	int g;
	cin >> g;
	map<string, int> group;
	for (int i = 0; i < g; i++) {
		string a, b, c;
		cin >> a >> b >> c;
		group[a] = group[b] = group[c] = i;
	}
	auto get = [&](const string &s) -> int {
		auto it = group.find(s);
		assert(it != group.end());
		return it->second;
	};
	int ans = 0;
	for (auto& [u, v] : same) {
		ans += get(u) != get(v);
	}
	for (auto& [u, v] : diff) {
		ans += get(u) == get(v);
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}