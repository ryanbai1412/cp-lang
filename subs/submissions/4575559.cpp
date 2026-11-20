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
	int n, q;
	cin >> n >> q;
	int arr[n][2], queries[q][2];
	vector<int> comp;
	auto add = [&](int x) -> void {
		comp.push_back(x);
		comp.push_back(x - 1);
		comp.push_back(x + 1);
	};
	for (auto& [l, r] : arr) {
		cin >> l >> r;
		add(l);
		add(r);
	}
	for (auto& [l, r] : queries) {
		cin >> l >> r;
		add(l);
		add(r);
	}
	sort(begin(comp), end(comp));
	comp.erase(unique(begin(comp), end(comp)), comp.end());
	auto cc = [&](int x) -> int {
		return int(lower_bound(begin(comp), end(comp), x) - comp.begin());
	};
	int m = sz(comp) + 10;
	int psum[m] {};
	for (auto& [l, r] : arr) {
		l = cc(l);
		r = cc(r);
		psum[l]++;
		psum[r + 1]--;
	}
	for (int i = 1; i < m; i++) {
		psum[i] += psum[i - 1];
	}
	for (auto& a : psum) {
		a = !!a;
	}
	vector<int> bad;
	for (int i = 0; i < m; i++) {
		if (!psum[i]) {
			bad.push_back(i);
			dbg(i);
		}
	}
	for (auto& [l, r] : queries) {
		l = cc(l);
		r = cc(r);
		auto it = lower_bound(begin(bad), end(bad), l);
		if (*it > r) {
			cout << "Y" << endl;
		} else {
			cout << "N" << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}