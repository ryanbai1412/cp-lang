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

	public void solve(Input in, PrintWriter pw) throws Exception {
		int n = in.nextInt();
		long[] dp = new long[n+1];
		dp[1] = 1;
		for(int i = 2; i<=n; i++) {
			for(int j = 2; j<=i; j++) {
				dp[i] += dp[i/j];
			}
		}
		pw.println(dp[n]);
	}
}