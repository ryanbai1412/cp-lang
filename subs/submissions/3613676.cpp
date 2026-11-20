#include "bits/extc++.h"

using namespace std;

#define sz(x) static_cast<int>((x).size())
#define long int64_t
#define endl "\n"

const int maxnm = 1e6 + 5;
const int dx[] {-1, 1, 0, 0}, dy[] {0, 0, -1, 1};

int n, m, ccomp, dist[maxnm];
vector<vector<int>> color, arr, vis;
vector<pair<int, int>> comp[maxnm], graph[maxnm];

inline bool ibs(int x, int y) {
	return 0 <= x && x < n && 0 <= y && y < m;
}

void dfs(int x, int y) {
	comp[ccomp].emplace_back(x, y);
	vis[x][y] = true;
	color[x][y] = ccomp;
	for(int i = 0; i < 4; i++) {
		int cx = x + dx[i];
		int cy = y + dy[i];
		if(ibs(cx, cy) && arr[cx][cy] && !vis[cx][cy]) {
			dfs(cx, cy);
		}
	}
}

void solve() {
	cin >> n >> m;
	arr.resize(n, vector<int>(m));
	color.resize(n, vector<int>(m));
	vis.resize(n, vector<int>(m));
	for(int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for(int j = 0; j < m; j++) {
			arr[i][j] = s[j] == '#';
		}
	}
	ccomp = 1;
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < m; j++) {
			if(arr[i][j] && !vis[i][j]) {
				dfs(i, j);
				ccomp++;
			}
		}
	}
	int lo[n + 1][m];
	memset(lo, -1, sizeof(lo));
	for(int i = 0; i < m; i++) {
		for(int j = n - 1; j >= 0; j--) {
			int p = lo[j + 1][i];
			if(arr[j][i]) {
				if(p == -1) {
					graph[0].emplace_back(color[j][i], n - j - 1);
				}else {
					graph[color[p][i]].emplace_back(color[j][i], p - j - 1);
				}
				p = j;
			}
			lo[j][i] = p;
		}
	}
	memset(dist, 0x7f, sizeof(dist));
	priority_queue<pair<int, int>, vector<pair<int, int>>, greater<>> pq;
	dist[0] = 0;
	pq.emplace(0, 0);
	while(sz(pq)) {
		auto [d, u] = pq.top();
		pq.pop();
		if(dist[u] == d) {
			for(auto &[v, w]: graph[u]) {
				if(d + w < dist[v]) {
					dist[v] = d + w;
					pq.emplace(dist[v], v);
				}
			}
		}
	}
	bool ans[n][m] {};
	for(int i = 1; i < ccomp; i++) {
		assert(dist[i] < 1e9);
		for(auto &[x, y]: comp[i]) {
			ans[x + dist[i]][y] = true;
		}
	}
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < m; j++) {
			cout << ".#"[ans[i][j]];
		}
		cout << endl;
	}
}

int main() {
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}