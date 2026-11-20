import java.util.*;
import java.io.*;

import static java.lang.Math.*;

public class Main {
	public static void main(String[] args) throws Exception {
		Input in = new Input(new InputStreamReader(System.in));
		PrintWriter out = new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.out), 1<<16));
		new Main().solve(in, out);
		in.close();
		out.close();
	}

	static class Input {
		BufferedReader br;
		StringTokenizer st;
		public Input(Reader reader) {
			br = new BufferedReader(reader, 1<<20);
			st = new StringTokenizer("");
		}
		public String next() throws IOException {
			while(!st.hasMoreTokens()) {
				st = new StringTokenizer(br.readLine());
			}
			return st.nextToken();
		}
		public int nextInt() throws IOException {
			return Integer.parseInt(next());
		}
		public void close() throws IOException {
			br.close();
		}
	}

	//	private final boolean local = System.getProperty("LOCAL")!=null;
	private final boolean local = false;

	public void dbg(Object... o) {
		if(local) {
			System.err.println(Arrays.deepToString(o));
		}
	}

	public void as(boolean flag) {
		if(!flag) {
			throw new AssertionError();
		}
	}

	static class State {
		int x, y, dist;
		public State(int x, int y, int dist) {
			this.x = x;
			this.y = y;
			this.dist = dist;
		}
	}

	final int[] dx = new int[] {0, 0, -1, 1}, dy = new int[] {-1, 1, 0, 0};
	final String dirs = "LRUD";
	public void solve(Input in, PrintWriter pw) throws Exception {
		int n = in.nextInt(), m = in.nextInt();
		char[][] arr = new char[n][];
		for(int i = 0; i<n; i++) {
			arr[i] = in.next().toCharArray();
		}
		int[][] dist = new int[n][m];
		boolean[][] can = new boolean[n][m];
		for(int i = 0; i<n; i++) {
			Arrays.fill(dist[i], -1);
			Arrays.fill(can[i], true);
		}
		int sx = -1, sy = -1;
		for(int i = 0; i<n; i++) {
			for(int j = 0; j<m; j++) {
				if(arr[i][j]=='C') {
					for(int k = 0; k<4; k++) {
						int x = i, y = j;
						while(arr[x][y]!='W') {
							if(dirs.indexOf(arr[x][y])==-1) {
								can[x][y] = false;
							}
							x += dx[k];
							y += dy[k];
						}
					}
				}else if(arr[i][j]=='W') {
					can[i][j] = false;
				}else if(arr[i][j]=='S') {
					sx = i;
					sy = j;
				}
			}
		}
		as(sx!=-1);
		if(can[sx][sy]) {
			ArrayDeque<State> q = new ArrayDeque<>();
			q.addLast(new State(sx, sy, 0));
			while(!q.isEmpty()) {
				State s = q.removeFirst();
				if(dist[s.x][s.y]==-1) {
					dist[s.x][s.y] = s.dist;
					if(dirs.indexOf(arr[s.x][s.y])!=-1) {
						int ind = dirs.indexOf(arr[s.x][s.y]);
						int cx = s.x+dx[ind], cy = s.y+dy[ind];
						if(can[cx][cy]&&dist[cx][cy]==-1) {
							q.addFirst(new State(cx, cy, s.dist));
						}
					}else {
						for(int i = 0; i<4; i++) {
							int cx = s.x+dx[i], cy = s.y+dy[i];
							if(can[cx][cy]&&dist[cx][cy]==-1) {
								q.addLast(new State(cx, cy, s.dist+1));
							}
						}
					}
				}
			}
		}
		for(int i = 0; i<n; i++) {
			for(int j = 0; j<m; j++) {
				if(arr[i][j]=='.') {
					pw.println(dist[i][j]);
				}
			}
		}
	}
}