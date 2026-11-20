#include "bits/extc++.h"

using namespace std;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

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

struct cowng {
	long x, a, b, c, d;

	cowng(int64_t x, int64_t a, int64_t b, int64_t c, int64_t d): x(x), a(a), b(b), c(c), d(d) {}

	long operator()() {
		long y = 0;
		y += c;
		y += (b * x) % d;
		y += (a * ((x * x) % d)) % d;
		y %= d;
		dbg(y);
		return x = y;
	}
};

void solve() {
	long rx, ra, rb, rc, rd;
	cin >> rx >> ra >> rb >> rc >> rd;
	cowng rng(rx, ra, rb, rc, rd);
	int n, m, q;
	cin >> n >> m >> q;
	int pos[n * m];
	iota(pos, pos + n * m, 0);
	pair<short, short> arr[n * m];
	int ind = 0;
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < m; j++) {
			arr[ind++] = {i, j};
		}
	}
	auto swp = [&] (int a, int b) {
		int &u = pos[a];
		int &v = pos[b];
		swap(arr[u], arr[v]);
		swap(u, v);
	};
	for(int i = 0; i < n * m; i++) {
		swp(i, rng() % (i + 1));
	}
	while(q--) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		swp(u, v);
	}
	map<int, short> top, bot;
	top[-1] = bot[-1] = 0;
	top[m] = bot[m] = static_cast<short>(n - 1);
	vector<int> ans;
	for(int i = 0; i < n * m; i++) {
		auto &[y, x] = arr[i];
		dbg(i + 1, x, y);
		if(y >= (--bot.lower_bound(x))->second && y <= (top.upper_bound(x)->second)) {
			ans.push_back(i);
			bot[x] = max(bot[x], y);
			auto it = top.find(x);
			if(it == top.end()) {
				top[x] = y;
			}else {
				it->second = min(it->second, y);
			}
		}
	}
	sort(begin(ans), end(ans));
	for(auto &a: ans) {
		cout << a + 1 << " ";
	}
	cout << endl;
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