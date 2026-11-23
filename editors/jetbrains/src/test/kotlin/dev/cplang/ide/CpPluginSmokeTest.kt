package dev.cplang.ide

import com.google.gson.JsonElement
import com.google.gson.JsonNull
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import com.intellij.execution.actions.ConfigurationContext
import com.intellij.openapi.actionSystem.IdeActions
import com.intellij.openapi.extensions.ExtensionPointName
import com.intellij.openapi.fileTypes.FileTypeManager
import com.intellij.openapi.fileTypes.SyntaxHighlighterFactory
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.newvfs.impl.VfsRootAccess
import com.intellij.xdebugger.ui.DebuggerColors
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import org.junit.Assume
import java.io.File
import java.nio.file.Files

/** The plugin as the IDE loads it: plugin.xml, the file type, highlighting, run configurations, the server and the trace. */
class CpPluginSmokeTest : BasePlatformTestCase() {
    private val repo = File(System.getProperty("cpl.repo"))
    private val cpl = File(System.getProperty("cpl.bin"))

    override fun setUp() {
        super.setUp()
        CplSettings.get().state.path = cpl.path
    }

    fun `test cp files are CP Lang and highlight`() {
        assertSame(CpFileType, FileTypeManager.getInstance().getFileTypeByFileName("a.cp"))
        myFixture.configureByText("a.cp", "fn main():\n    print(1)\n")
        assertSame(CpLanguage, myFixture.file.language)
        val highlighter = SyntaxHighlighterFactory.getSyntaxHighlighter(CpLanguage, project, myFixture.file.virtualFile)
        assertTrue(highlighter is CpSyntaxHighlighter)
        assertEquals(CpColors.KEYWORD, highlighter.getTokenHighlights(CpTokens.KEYWORD).single())
    }

    fun `test line comments toggle with two slashes`() {
        myFixture.configureByText("a.cp", "<caret>let x = 1\n")
        myFixture.performEditorAction(IdeActions.ACTION_COMMENT_LINE)
        myFixture.checkResult("// let x = 1\n")
    }

    fun `test the language server is registered`() {
        val providers = ExtensionPointName.create<Any>("com.intellij.platform.lsp.serverSupportProvider").extensionList
        assertTrue(providers.any { it is CpLspServerSupportProvider })
        val cmd = CpLspServerDescriptor(project).createCommandLine()
        assertEquals(listOf(cpl.path, "lsp"), cmd.getCommandLineList(null))
    }

    fun `test a cp file makes a run configuration on its first sample`() {
        val dir = Files.createTempDirectory("cpl").toFile()
        // macOS's temp folder sits outside the roots a test may read.
        VfsRootAccess.allowRootAccess(testRootDisposable, dir.canonicalPath)
        File(dir, "i1").writeText("3\n")
        val source = File(dir, "sum.cp").apply { writeText("fn main():\n    let n: Int = read()\n    print(n + 1)\n") }
        val vfile = LocalFileSystem.getInstance().refreshAndFindFileByIoFile(source)!!
        myFixture.configureFromExistingVirtualFile(vfile)
        val context = ConfigurationContext(myFixture.file)
        val made = context.configurationsFromContext.orEmpty().map { it.configuration }.filterIsInstance<CplRunConfiguration>()
        val config = made.single()
        // The IDE spells paths with `/` and through the real temp folder on every OS.
        val same = { path: String -> File(path).canonicalPath }
        assertEquals(same(source.path), same(config.options.file!!))
        assertEquals("run", config.options.command)
        assertEquals("run sum.cp", config.name)
        assertEquals(
            listOf(cpl.path, "run", source.path, File(dir, "i1").path).map(same),
            config.commandLine().getCommandLineList(null).map(same),
        )
    }

    fun `test the stress configuration saves a failing case that the test configuration reruns`() {
        val dir = Files.createTempDirectory("cpl").toFile()
        val planted = File(repo, "examples/debugging/knapsack.cp").readText()
        assertTrue(planted.contains("for c in 0..=cap:"))
        val source = File(dir, "k.cp").apply { writeText(planted) }
        fun run(command: String): Pair<Int, String> {
            val config = CplRunConfigurationType.get().configurationFactories.single()
                .createTemplateConfiguration(project) as CplRunConfiguration
            config.options.file = source.path
            config.options.command = command
            config.checkConfiguration()
            val process = config.commandLine().toProcessBuilder().redirectErrorStream(true).start()
            val shown = process.inputStream.bufferedReader().readText()
            return process.waitFor() to shown
        }
        val (stressed, report) = run("stress")
        assertEquals(report, 1, stressed)
        assertTrue(report, report.contains("saved"))
        assertEquals(report, 1, File(dir, "k.tests").listFiles { f -> f.name.endsWith(".in") }.orEmpty().size)
        val (failed, rerun) = run("test")
        assertTrue(rerun, failed != 0)
        source.writeText(planted.replace("for c in 0..=cap:", "for c in (0..=cap).rev:"))
        val (passed, fixed) = run("test")
        assertEquals(fixed, 0, passed)
    }

    fun `test fn main has a run marker`() {
        myFixture.configureByText("a.cp", "fn helper():\n    pass\n\nfn ma<caret>in():\n    print(1)\n")
        val element = myFixture.file.findElementAt(myFixture.file.text.indexOf("fn main"))!!
        assertNotNull(CplRunLineMarkerContributor().getInfo(element))
        val helper = myFixture.file.findElementAt(0)!!
        assertNull(CplRunLineMarkerContributor().getInfo(helper))
    }

    fun `test the server answers initialize and hover`() {
        Assume.assumeTrue("build cpl first: cargo build -p cpl-cli", cpl.canExecute())
        val process = CpLspServerDescriptor(project).createCommandLine().createProcess()
        try {
            val io = LspPipe(process)
            io.send("""{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}""")
            val init = io.receive(1)
            assertTrue(init, init.contains("\"hoverProvider\":true"))
            io.send("""{"jsonrpc":"2.0","method":"initialized","params":{}}""")
            val text = "fn main():\\n    let n: Int = read()\\n    print(n + 1)\\n"
            io.send("""{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/a.cp","languageId":"cp","version":1,"text":"$text"}}}""")
            io.send("""{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/a.cp"},"position":{"line":2,"character":10}}}""")
            val hover = io.receive(2)
            assertTrue(hover, hover.contains("n: Int"))
            io.send("""{"jsonrpc":"2.0","id":3,"method":"shutdown"}""")
            io.receive(3)
        } finally {
            process.destroy()
        }
    }

    /** Every request of editors/lsp-matrix.json, sent to the server the plugin starts. */
    fun `test every request of the lsp matrix answers`() {
        Assume.assumeTrue("build cpl first: cargo build -p cpl-cli", cpl.canExecute())
        val matrix = JsonParser.parseString(File(repo, "editors/lsp-matrix.json").readText()).asJsonObject
        val dir = Files.createTempDirectory("cpl-matrix").toFile()
        for ((name, body) in matrix.getAsJsonObject("files").entrySet()) File(dir, name).writeText(body.asString)
        val main = File(dir, "main.cp")
        val uri = main.toPath().toUri().toString()
        val process = CpLspServerDescriptor(project).createCommandLine().createProcess()
        try {
            val io = LspPipe(process)
            io.send("""{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}""")
            io.receive(1)
            io.send("""{"jsonrpc":"2.0","method":"initialized","params":{}}""")
            val open = JsonObject().apply {
                add("textDocument", JsonObject().apply {
                    addProperty("uri", uri)
                    addProperty("languageId", "cp")
                    addProperty("version", 1)
                    addProperty("text", main.readText())
                })
            }
            io.send("""{"jsonrpc":"2.0","method":"textDocument/didOpen","params":$open}""")
            val results = mutableMapOf<String, JsonElement>()
            fun earlier(value: JsonElement): JsonElement = when {
                value.isJsonPrimitive && value.asJsonPrimitive.isString && value.asString.startsWith("\$result:") -> {
                    val rest = value.asString.removePrefix("\$result:")
                    val found = results.getValue(rest.substringBeforeLast(':'))
                    val key = rest.substringAfterLast(':')
                    key.toIntOrNull()?.let { found.asJsonArray[it] } ?: found.asJsonObject[key]
                }
                value.isJsonObject -> JsonObject().also { out ->
                    for ((k, v) in value.asJsonObject.entrySet()) out.add(k, earlier(v))
                }
                else -> value
            }
            val requests = matrix.getAsJsonArray("requests")
            for ((i, case) in requests.withIndex()) {
                val method = case.asJsonObject["method"].asString
                val params = earlier(case.asJsonObject["params"]).asJsonObject
                params.add("textDocument", JsonObject().apply { addProperty("uri", uri) })
                val id = 100 + i
                io.send("""{"jsonrpc":"2.0","id":$id,"method":"$method","params":$params}""")
                val reply = JsonParser.parseString(io.receive(id)).asJsonObject
                val result = reply["result"] ?: JsonNull.INSTANCE
                results[method] = result
                val got = result.toString()
                val missing = case.asJsonObject["expect"].asJsonArray.map { it.asString }.filter { !got.contains(it) }
                assertTrue("$method lacks $missing: $reply", reply["error"] == null && missing.isEmpty())
            }
            io.send("""{"jsonrpc":"2.0","id":2,"method":"shutdown"}""")
            io.receive(2)
        } finally {
            process.destroy()
        }
    }

    fun `test the trace view renders`() {
        Assume.assumeTrue("build cpl first: cargo build -p cpl-cli", cpl.canExecute())
        val dir = Files.createTempDirectory("cpl").toFile()
        val source = File(dir, "sum.cp").apply { writeText("var total = 0\n\nfn main():\n    let n: Int = read()\n    for i in 0..n:\n        total += i\n    print(total)\n") }
        val input = File(dir, "i1").apply { writeText("4\n") }
        val html = CplTrace.render(source, null, input)
        assertTrue(html.take(200), Regex("^\\s*<(!doctype|html)", RegexOption.IGNORE_CASE).containsMatchIn(html))
        assertTrue(html.contains("total"))
        val failed = runCatching { CplTrace.render(File(dir, "missing.cp"), null, null) }.exceptionOrNull()
        assertTrue(failed is CplTrace.Failed)
        val bridged = CplTrace.bridge(html, "post(json);")
        assertTrue(bridged.indexOf("window.acquireVsCodeApi") < bridged.indexOf("<script id=\"trace-ui\">"))
        assertTrue("the viewer posts through acquireVsCodeApi", html.contains("acquireVsCodeApi()"))
    }

    fun `test the editor marks the trace's step and a clicked line moves the caret`() {
        myFixture.configureByText("a.cp", "<caret>fn main():\n    let x = 1\n    print(x)\n")
        val follower = CplTraceFollower(project, myFixture.file.virtualFile)
        Disposer.register(testRootDisposable, follower)
        fun marked() = myFixture.editor.markupModel.allHighlighters
            .filter { it.isValid && it.textAttributesKey == DebuggerColors.EXECUTIONPOINT_ATTRIBUTES }
            .map { myFixture.editor.document.getLineNumber(it.startOffset) }
        follower.follow(TraceMessage(false, 2))
        assertEquals(listOf(1), marked())
        assertEquals(0, myFixture.editor.caretModel.logicalPosition.line)
        follower.follow(TraceMessage(true, 3))
        assertEquals(listOf(2), marked())
        assertEquals(2, myFixture.editor.caretModel.logicalPosition.line)
        follower.follow(TraceMessage(false, 99))
        assertEquals(listOf(3), marked())
        Disposer.dispose(follower)
        assertEquals(emptyList<Int>(), marked())
    }
}

private class LspPipe(private val process: Process) {
    private val input = process.inputStream.buffered()

    fun send(json: String) {
        val bytes = json.toByteArray(Charsets.UTF_8)
        process.outputStream.write("Content-Length: ${bytes.size}\r\n\r\n".toByteArray(Charsets.US_ASCII))
        process.outputStream.write(bytes)
        process.outputStream.flush()
    }

    /** The reply to request `id`, skipping notifications such as diagnostics. */
    fun receive(id: Int): String {
        while (true) {
            var length = -1
            while (true) {
                val line = StringBuilder()
                while (true) {
                    val c = input.read()
                    check(c >= 0) { "cpl lsp closed its output" }
                    if (c == '\n'.code) break
                    if (c != '\r'.code) line.append(c.toChar())
                }
                if (line.isEmpty()) break
                if (line.startsWith("Content-Length:")) length = line.substring(15).trim().toInt()
            }
            val body = String(input.readNBytes(length), Charsets.UTF_8)
            if (body.contains("\"id\":$id,") || body.contains("\"id\":$id}")) return body
        }
    }
}
