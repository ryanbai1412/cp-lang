#include "bits/extc++.h"

using namespace std;

template<class T, class U>
void sep(T &out, const string &s, const U &u) {
	out << u;
}

template<class T, class Head, class ...Tail>
void sep(T &out, const string &s, const Head &h, const Tail &...t) {
	out << h << s;
	sep(out, s, t...);
}

#ifdef DEBUG
#define dbg(...)                                                      \
cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": ";       \
sep(cerr, " | ", __VA_ARGS__);                                        \
cerr << endl
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

void solve() {
	int n, k;
	cin >> n >> k;
	vector<char> bad[n] {};
	for(auto &a: bad) {
		a.resize(n);
	}
	for(int i = 0; i < k; i++) {
		int x, y;
		cin >> x >> y;
		bad[x - 1][y - 1] = true;
	}
	vector<int> from[n];
	for(auto &a: from) {
		a.resize(n, -1);
	}
	deque<pair<int, int>> q;
	q.emplace_back(0, 0);
	auto valid = [&] (int x, int y) {
		return max(x, y) < n && !bad[x][y] && from[x][y] == -1;
	};
	while(sz(q)) {
		auto [x, y] = q.front();
		q.pop_front();
		if(valid(x, y + 1)) {
			from[x][y + 1] = 0;
			q.emplace_back(x, y + 1);
		}
		if(valid(x + 1, y)) {
			from[x + 1][y] = 1;
			q.emplace_back(x + 1, y);
		}
	}
	if(from[n - 1][n - 1] == -1) {
		cout << "IMPOSSIBLE" << endl;
	}else {
		vector<int> ans;
		int x = n - 1, y = n - 1;
		while(from[x][y] != -1) {
			ans.push_back(from[x][y]);
			if(from[x][y] == 0) {
				y--;
			}else {
				x--;
			}
		}
		assert(x == 0 && y == 0);
		reverse(begin(ans), end(ans));
		for(auto &a: ans) {
			cout << "RU"[a];
		}
		cout << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}