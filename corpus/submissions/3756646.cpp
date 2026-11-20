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

const int maxn = 1000;

int n, edges[maxn][2];
vector<int> graph[maxn];

void line() {
	if(n == 1) {
		cout << 1 << endl;
		cout << 1 << endl;
		cout << "L 1" << endl;
		return;
	}
	int deg[n] {};
	for(int i = 0; i < n; i++) {
		auto &[u, v] = edges[i];
		deg[u]++;
		deg[v]++;
	}
	assert(count(deg, deg + n, 1) == 2);
	cout << 1 << endl;
	cout << n << endl;
	bool vis[n] {};
	int u = find(deg, deg + n, 1) - deg;
	cout << "L " << u + 1 << endl;
	while(true) {
		vis[u] = true;
		int next = -1;
		for(auto &v: graph[u]) {
			if(!vis[v]) {
				assert(next == -1);
				next = v;
			}
		}
		if(next == -1) {
			break;
		}
		cout << "M " << u + 1 << " " << next + 1 << endl;
		u = next;
	}
}

void dumb() {
	cout << n + n - 1 << endl;
	cout << n + 2 * (n - 1) << endl;
	for(int i = 0; i < n; i++) {
		cout << "L " << i + 1 << endl;
	}
	for(int i = 0; i < n - 1; i++) {
		auto &[u, v] = edges[i];
		cout << "L " << u + 1 << endl;
		cout << "M " << u + 1 << " " << v + 1 << endl;
	}
}

void solve() {
	cin >> n;
	int deg[n] {};
	for(int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
		deg[u]++;
		deg[v]++;
		auto &[_u, _v] = edges[i];
		_u = u;
		_v = v;
	}
	if(*max_element(deg, deg + n) <= 2) {
		line();
	}else {
		dumb();
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