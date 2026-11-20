#include "bits/extc++.h"

using namespace std;

template<class T>
void dbgh(T t) {
	cerr << t << endl;
}

template<class T, class... U>
void dbgh(T t, U... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; dbgh(__VA_ARGS__); cerr
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

#define long int64_t
#define sz(x) static_cast<int>((x).size())

void solve() {
	int n, m;
	cin >> n >> m;
	set<int> graph[n];
	for(int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].insert(v);
	}
	long ans = 0;
	for(int i = 0; i < n; i++) {
		graph[i].erase(i);
		ans += sz(graph[i]);
		if(sz(graph[i])) {
			int f = *graph[i].begin();
			if(sz(graph[f]) > sz(graph[i])) {
				swap(graph[i], graph[f]);
			}
			graph[f].insert(begin(graph[i]), end(graph[i]));
		}
	}
	cout << ans << endl;
}

int main() {
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}