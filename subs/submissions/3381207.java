//package Mock_CCC_Senior21;

import java.util.*;
import java.io.*;

import static java.lang.Math.*;

public class Problem4 {
	static class Input {
		BufferedReader br = new BufferedReader(new InputStreamReader(System.in), 1<<16);
		StringTokenizer st = new StringTokenizer("");
		public String next() throws IOException {
			while(!st.hasMoreTokens()) {
				st = new StringTokenizer(br.readLine());
			}
			return st.nextToken();
		}
		public int nextInt() throws IOException {
			return Integer.parseInt(next());
		}
	}

	public static void main(String[] args) throws Exception {
		Input in = new Input();
		PrintWriter pw = new PrintWriter(System.out);
		new Problem4().solve(in, pw);
		pw.close();
	}

	final int n = 53;
	int ans;
	int[] visited;
	ArrayList<Integer>[] graph;
	public void dfs(int u) {
		if(visited[u]!=0) {
			if(visited[u]==1) {
				ans = 1;
			}
			return;
		}
		visited[u] = 1;
		for(int v: graph[u]) {
			dfs(52-v);
		}
		visited[u] = 2;
	}
	public void solve(Input in, PrintWriter pw) throws Exception {
		int m = in.nextInt();
		graph = new ArrayList[n];
		for(int i = 0; i<n; i++) {
			graph[i] = new ArrayList<>();
		}
		for(int i = 0; i<m; i++) {
			int[] cur = new int[4];
			for(int j = 0; j<4; j++) {
				cur[j] = in.nextInt()+26;
			}
			for(int j = 0; j<4; j++) {
				if(cur[j]!=26) {
					for(int k = 0; k<4; k++) {
						if(j!=k&&cur[k]!=26) {
							graph[cur[j]].add(cur[k]);
						}
					}
				}
			}
		}
		ans = 0;
		visited = new int[n];
		for(int i = 0; i<n; i++) {
			dfs(i);
		}
		pw.println(ans);
	}
}