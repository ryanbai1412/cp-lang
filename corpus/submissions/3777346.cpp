#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

struct Compressor {
	int ind = 0;
	map<int, int> m;

	int operator()(int x) {
		int &ans = m[x];
		if(!ans) {
			ans = ++ind;
		}
		return ans - 1;
	}
};

struct Tree {
	int x, y, i;
};

struct Edge {
	int v, find, tind;
};

const int maxn = 1e6;

int memo[maxn];
Tree arr[maxn];
Compressor compr;
vector<int> hlines[maxn];
vector<Edge> graph[maxn];

void addEdge(int u, int v) {
	dbg("EDGE", arr[u].i, arr[v].i);
	int uy = arr[u].y, vy = arr[v].y;
	if(uy != vy) {
		if(uy > vy) {
			swap(uy, vy);
			swap(u, v);
		}
		graph[compr(uy)].push_back({compr(vy), arr[u].i, arr[v].i});
	}
}

void addEdge(int i) {
	addEdge(i, i + 1);
}

int dp(int u) {
	int &ans = memo[u];
	if(ans != -1) {
		return ans;
	}
	ans = 0;
	for(auto &[v, _, __]: graph[u]) {
		ans = max(ans, dp(v));
		dbg(u, v, dp(v));
	}
	ans += sz(hlines[u]);
	return ans;
}

void solve() {
	int n;
	cin >> n;
	for(int i = 0; i < n; i++) {
		cin >> arr[i].x >> arr[i].y;
		arr[i].i = i;
	}
	arr[n] = {0, 0, n};
	n++;

	for(int i = 0; i < n; i++) {
		hlines[compr(arr[i].y)].push_back(arr[i].i);
	}

	auto process = [&](auto cmp) -> void {
		sort(arr, arr + n, [&](const Tree &a, const Tree &b) -> bool {
			int cmpa = cmp(a);
			int cmpb = cmp(b);
			return tie(cmpa, a.y) < tie(cmpb, b.y);
		});
		for(int i = 0; i < n - 1; i++) {
			if(cmp(arr[i]) == cmp(arr[i + 1])) {
				addEdge(i);
			}
		}
	};

	// up
	process([&](const Tree &x) -> int {
		return x.x;
	});

	// up left
	process([&](const Tree &x) -> int {
		return x.x + x.y;
	});

	// up right
	process([&](const Tree &x) -> int {
		return x.y - x.x;
	});

	memset(memo, -1, sizeof(memo));
	int ans = dp(compr(0)) - 1;
	cout << ans << endl;
	for(int i = 0; i < ans; i++) {
		cout << 0 << " ";
	}
	cout << 0 << endl;
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