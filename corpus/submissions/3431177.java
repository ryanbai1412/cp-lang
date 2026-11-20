import java.io.*;
import java.util.Arrays;
import java.util.StringTokenizer;

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

	int n;
	int[][] arr;
	public void rotate() {
		int[][] next = new int[n][n];
		for(int i = 0; i<(n+1)/2; i++) {
			for(int j = 0; j<(n+1)/2; j++) {
				int x = i, y = j;
				for(int k = 0; k<4; k++) {
					int cur = arr[x][y];
					int tmp = x;
					x = y;
					y = n-1-tmp;
					next[x][y] = cur;
				}
			}
		}
		arr = next;
	}
	public boolean solve() {
		for(int i = 0; i<n; i++) {
			int prev = -1;
			for(int j = 0; j<n; j++) {
				if(arr[j][i]<=prev) {
					return false;
				}
				prev = arr[j][i];
			}
		}
		for(int i = 0; i<n; i++) {
			int prev = -1;
			for(int j = 0; j<n; j++) {
				if(arr[i][j]<=prev) {
					return false;
				}
				prev = arr[i][j];
			}
		}
		return true;
	}
	public void solve(Input in, PrintWriter pw) throws Exception {
		n = in.nextInt();
		arr = new int[n][n];
		for(int i = 0; i<n; i++) {
			for(int j = 0; j<n; j++) {
				arr[i][j] = in.nextInt();
			}
		}
		for(int i = 0; i<4; i++) {
			rotate();
			if(solve()) {
				for(int j = 0; j<n; j++) {
					for(int k = 0; k<n; k++) {
						pw.print(arr[j][k]+" ");
					}
					pw.println();
				}
				return;
			}
		}
		as(false);
	}
}