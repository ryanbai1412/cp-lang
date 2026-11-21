package dev.cplang.ide

import com.intellij.execution.ExecutionException
import com.intellij.execution.process.CapturingProcessHandler
import com.intellij.ide.BrowserUtil
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.impl.HTMLEditorProvider
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.ui.jcef.JBCefApp
import java.io.File

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

    fun show(project: Project, title: String, html: String, source: File) {
        if (JBCefApp.isSupported()) {
            HTMLEditorProvider.openEditor(project, title, html)
        } else {
            val page = File(source.absoluteFile.parentFile, "${source.name}.trace.html")
            page.writeText(html)
            BrowserUtil.browse(page)
        }
    }
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
