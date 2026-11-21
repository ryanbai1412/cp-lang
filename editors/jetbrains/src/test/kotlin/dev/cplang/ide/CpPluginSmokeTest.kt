package dev.cplang.ide

import com.intellij.execution.actions.ConfigurationContext
import com.intellij.openapi.actionSystem.IdeActions
import com.intellij.openapi.extensions.ExtensionPointName
import com.intellij.openapi.fileTypes.FileTypeManager
import com.intellij.openapi.fileTypes.SyntaxHighlighterFactory
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
        File(dir, "i1").writeText("3\n")
        val source = File(dir, "sum.cp").apply { writeText("fn main():\n    let n: Int = read()\n    print(n + 1)\n") }
        val vfile = myFixture.copyFileToProject(source.path, "sum.cp")
        File(File(vfile.path).parentFile, "i1").writeText("3\n")
        myFixture.configureFromExistingVirtualFile(vfile)
        val context = ConfigurationContext(myFixture.file)
        val made = context.configurationsFromContext.orEmpty().map { it.configuration }.filterIsInstance<CplRunConfiguration>()
        val config = made.single()
        assertEquals(vfile.path, config.options.file)
        assertEquals("run", config.options.command)
        assertEquals("run sum.cp", config.name)
        assertEquals(listOf(cpl.path, "run", vfile.path, File(File(vfile.path).parentFile, "i1").path), config.commandLine().getCommandLineList(null))
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
