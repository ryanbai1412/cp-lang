package dev.cplang.ide

import com.intellij.execution.ExecutionException
import com.intellij.execution.process.CapturingProcessHandler
import com.google.gson.JsonParser
import com.intellij.ide.BrowserUtil
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.editor.LogicalPosition
import com.intellij.openapi.editor.ScrollType
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.RangeHighlighter
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.OpenFileDescriptor
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.wm.ToolWindowAnchor
import com.intellij.openapi.wm.ToolWindowManager
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.jcef.JBCefApp
import com.intellij.ui.jcef.JBCefBrowser
import com.intellij.ui.jcef.JBCefBrowserBase
import com.intellij.ui.jcef.JBCefJSQuery
import com.intellij.xdebugger.ui.DebuggerColors
import java.awt.BorderLayout
import java.io.File
import javax.swing.JPanel

object CplTrace {
    class Failed(message: String) : Exception(message)

    /** `cpl run FILE --trace [FN] --format html` on `input`: the trace viewer, one self-contained page. */
    fun render(source: File, fn: String?, input: File?): String {
        val cmd = Cpl.commandLine(Cpl.traceArgs(source.path, fn), source.absoluteFile.parentFile)
        if (input != null) cmd.withInput(input)
        val out = CapturingProcessHandler(cmd).runProcess(120_000)
        if (out.isTimeout) throw Failed("cpl run --trace took over 2 minutes")
        if (!Regex("^\\s*<(!doctype|html)", RegexOption.IGNORE_CASE).containsMatchIn(out.stdout)) {
            throw Failed(out.stderr.ifBlank { out.stdout }.ifBlank { "cpl exited with ${out.exitCode}" })
        }
        return out.stdout
    }

    const val TOOL_WINDOW = "CP Lang Trace"

    /**
     * `html` with the `acquireVsCodeApi` the viewer looks for, so it posts its step and clicked
     * lines through `post`, JS that sends the string `json`.
     */
    fun bridge(html: String, post: String): String {
        val shim = "<script>window.acquireVsCodeApi = () => ({ postMessage: message => { const json = JSON.stringify(message); $post } });</script>"
        val head = Regex("<head[^>]*>", RegexOption.IGNORE_CASE).find(html) ?: return shim + html
        return html.substring(0, head.range.last + 1) + shim + html.substring(head.range.last + 1)
    }

    /** One trace tab per file in the trace tool window, reused by the next trace of that file. */
    fun show(project: Project, title: String, html: String, source: File) {
        val file = LocalFileSystem.getInstance().refreshAndFindFileByIoFile(source)
        if (JBCefApp.isSupported() && file != null) {
            val manager = ToolWindowManager.getInstance(project)
            val window = manager.getToolWindow(TOOL_WINDOW) ?: manager.registerToolWindow(TOOL_WINDOW) {
                anchor = ToolWindowAnchor.RIGHT
                canCloseContent = true
            }
            val contents = window.contentManager
            var content = contents.contents.firstOrNull { (it.component as? CplTracePanel)?.file == file }
            if (content == null) {
                val panel = CplTracePanel(project, file)
                content = ContentFactory.getInstance().createContent(panel, title, false)
                content.setDisposer(panel)
                contents.addContent(content)
            }
            content.displayName = title
            (content.component as CplTracePanel).load(html)
            contents.setSelectedContent(content)
            window.show()
        } else {
            val page = File(source.absoluteFile.parentFile, "${source.name}.trace.html")
            page.writeText(html)
            BrowserUtil.browse(page)
        }
    }
}

/** What the trace viewer posts: the step it shows is on `line`, or (`reveal`) the user clicked `line`. */
data class TraceMessage(val reveal: Boolean, val line: Int) {
    companion object {
        fun parse(json: String): TraceMessage? {
            val obj = runCatching { JsonParser.parseString(json).asJsonObject }.getOrNull() ?: return null
            val type = obj.get("type")?.takeIf { it.isJsonPrimitive }?.asString
            val line = obj.get("line")?.takeIf { it.isJsonPrimitive && it.asJsonPrimitive.isNumber }?.asDouble ?: return null
            if (type != "step" && type != "reveal" || line < 1 || line != Math.floor(line)) return null
            return TraceMessage(type == "reveal", line.toInt())
        }
    }
}

/** Marks the line of the step the trace shows in `file`, like the debugger's execution point; a click moves the caret there. */
class CplTraceFollower(private val project: Project, private val file: VirtualFile) : Disposable {
    private var shown: RangeHighlighter? = null

    fun follow(message: TraceMessage) {
        val editor = FileEditorManager.getInstance(project).openTextEditor(OpenFileDescriptor(project, file), message.reveal) ?: return
        val document = editor.document
        val line = (message.line - 1).coerceIn(0, maxOf(document.lineCount - 1, 0))
        clear()
        shown = editor.markupModel.addLineHighlighter(DebuggerColors.EXECUTIONPOINT_ATTRIBUTES, line, HighlighterLayer.SELECTION - 1)
        if (message.reveal) editor.caretModel.moveToLogicalPosition(LogicalPosition(line, 0))
        editor.scrollingModel.scrollTo(LogicalPosition(line, 0), ScrollType.MAKE_VISIBLE)
    }

    private fun clear() {
        shown?.takeIf { it.isValid }?.dispose()
        shown = null
    }

    override fun dispose() = clear()
}

/** The trace viewer in JCEF, wired to its file through [CplTrace.bridge]. */
class CplTracePanel(project: Project, val file: VirtualFile) : JPanel(BorderLayout()), Disposable {
    private val browser = JBCefBrowser()
    private val query = JBCefJSQuery.create(browser as JBCefBrowserBase)
    private val follower = CplTraceFollower(project, file)

    init {
        Disposer.register(this, browser)
        Disposer.register(this, query)
        Disposer.register(this, follower)
        query.addHandler { json ->
            TraceMessage.parse(json)?.let { message ->
                ApplicationManager.getApplication().invokeLater({ follower.follow(message) }, project.disposed)
            }
            null
        }
        add(browser.component, BorderLayout.CENTER)
    }

    fun load(html: String) = browser.loadHTML(CplTrace.bridge(html, query.inject("json")))

    override fun dispose() {}
}

/** Run the file on its first sample with `--trace` and open the trace viewer, scoped to the fn at the caret. */
class CplShowTraceAction : AnAction() {
    override fun getActionUpdateThread() = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.getData(CommonDataKeys.VIRTUAL_FILE)?.extension == "cp"
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = e.getData(CommonDataKeys.VIRTUAL_FILE)?.takeIf { it.extension == "cp" } ?: return
        val editor = e.getData(CommonDataKeys.EDITOR)
        val fn = editor?.let { Cpl.enclosingFn(it.document.charsSequence, it.caretModel.offset) }?.takeIf { it != "main" }
        FileDocumentManager.getInstance().saveAllDocuments()
        val source = File(file.path)
        val input = Cpl.samples(source).firstOrNull()
        val title = "Trace: ${file.name}" + (fn?.let { " ($it)" } ?: "") + (input?.let { " on ${it.name}" } ?: "")
        object : Task.Backgroundable(project, "cpl run --trace", true) {
            override fun run(indicator: ProgressIndicator) {
                try {
                    val html = CplTrace.render(source, fn, input)
                    ApplicationManager.getApplication().invokeLater { CplTrace.show(project, title, html, source) }
                } catch (err: Exception) {
                    if (err !is CplTrace.Failed && err !is ExecutionException) throw err
                    NotificationGroupManager.getInstance().getNotificationGroup("CP Lang")
                        .createNotification("No trace", err.message ?: "", NotificationType.ERROR)
                        .notify(project)
                }
            }
        }.queue()
    }
}
