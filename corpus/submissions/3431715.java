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

	long[] dp;
	public long dp(int x) {
		if(dp[x]!=-1) {
			return dp[x];
		}
		dp[x] = 0;
		for(int i = 2; i<=x; i++) {
			dp[x] += dp(x/i);
		}
		return dp[x];
	}
	public void solve(Input in, PrintWriter pw) throws Exception {
		int n = in.nextInt();
		dp = new long[n+1];
		Arrays.fill(dp, -1);
		dp[1] = 1;
		pw.println(dp(n));
	}
}