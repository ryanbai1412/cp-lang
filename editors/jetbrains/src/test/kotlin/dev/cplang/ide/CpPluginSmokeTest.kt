package dev.cplang.ide

import com.google.gson.JsonElement
import com.google.gson.JsonNull
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.actionSystem.IdeActions
import com.intellij.openapi.extensions.ExtensionPointName
import com.intellij.openapi.fileTypes.FileTypeManager
import com.intellij.openapi.fileTypes.SyntaxHighlighterFactory
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.newvfs.impl.VfsRootAccess
import com.intellij.xdebugger.ui.DebuggerColors
import com.intellij.testFramework.PlatformTestUtil
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

    fun `test the server runs the cpl settings name, else the one on the PATH`() {
        assertEquals(listOf(cpl.path, "lsp"), Cpl.commandLine(Cpl.lspArgs(), null).getCommandLineList(null))
        CplSettings.get().state.path = " "
        assertEquals("cpl", Cpl.commandLine(Cpl.lspArgs(), null).exePath)
    }

    fun `test a cp file in LightEdit says it gets no language server`() {
        val notice = com.intellij.ui.EditorNotificationProvider.EP_NAME.getExtensions(project)
            .filterIsInstance<CpLightEditNotice>().single()
        assertTrue(notice is com.intellij.ide.lightEdit.LightEditCompatible)
        myFixture.configureByText("a.cp", "print(1)\n")
        assertNull(notice.collectNotificationData(project, myFixture.file.virtualFile))
        assertTrue(CpLightEditNotice.MESSAGE.contains("no diagnostics"))
    }

    fun `test cp files are CP Lang and highlight`() {
        assertSame(CpFileType, FileTypeManager.getInstance().getFileTypeByFileName("a.cp"))
        myFixture.configureByText("a.cp", "fn main():\n    print(1)\n")
        assertSame(CpLanguage, myFixture.file.language)
        val highlighter = SyntaxHighlighterFactory.getSyntaxHighlighter(CpLanguage, project, myFixture.file.virtualFile)
        assertTrue(highlighter is CpSyntaxHighlighter)
        assertEquals("CP_KEYWORD", highlighter.getTokenHighlights(CpTokens.named("KEYWORD")).single().externalName)
    }

    fun `test line comments toggle with two slashes`() {
        myFixture.configureByText("a.cp", "<caret>let x = 1\n")
        myFixture.performEditorAction(IdeActions.ACTION_COMMENT_LINE)
        myFixture.checkResult("// let x = 1\n")
    }

    fun `test the plugin requires lsp4ij and runs cpl lsp`() {
        val plugin = com.intellij.ide.plugins.PluginManagerCore.getPlugin(com.intellij.openapi.extensions.PluginId.getId("dev.cplang.ide"))!!
        val lsp4ij = plugin.dependencies.single { it.pluginId.idString == "com.redhat.devtools.lsp4ij" }
        assertFalse("LSP4IJ is required, not optional", lsp4ij.isOptional)
        val providers = ExtensionPointName.create<Any>("com.intellij.platform.lsp.serverSupportProvider").extensionList
        assertTrue("no second client starts `cpl lsp`", providers.none { it.javaClass.name.startsWith("dev.cplang") })
        assertEquals(listOf(cpl.path, "lsp"), Cpl.commandLine(Cpl.lspArgs(), null).getCommandLineList(null))
    }

    fun `test the trace window is declared and hidden until a trace opens`() {
        val window = com.intellij.openapi.wm.ToolWindowEP.EP_NAME.extensionList.single { it.id == "CP Lang Trace" }
        assertEquals("dev.cplang.ide.CplTraceWindow", window.factoryClass)
        assertFalse(CplTraceWindow().shouldBeAvailable(project))
    }

    fun `test the server answers initialize and hover`() {
        Assume.assumeTrue("build cpl first: cargo build -p cpl-cli", cpl.canExecute())
        val process = Cpl.commandLine(Cpl.lspArgs(), null).createProcess()
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
        val process = Cpl.commandLine(Cpl.lspArgs(), null).createProcess()
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

    fun `test the editor marks the line the trace page posts`() {
        myFixture.configureByText("a.cp", "<caret>fn main():\n    let x = 1\n    print(x)\n")
        fun marked() = myFixture.editor.markupModel.allHighlighters
            .filter { it.isValid && it.textAttributesKey == DebuggerColors.EXECUTIONPOINT_ATTRIBUTES }
            .map { myFixture.editor.document.getLineNumber(it.startOffset) }
        val first = CplTrace.follow(project, myFixture.file.virtualFile, 1, null)
        assertEquals(listOf(1), marked())
        val second = CplTrace.follow(project, myFixture.file.virtualFile, 2, first)
        assertEquals(listOf(2), marked())
        assertEquals(0, myFixture.editor.caretModel.logicalPosition.line)
        second.dispose()
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
