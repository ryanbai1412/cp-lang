package dev.cplang.ide

import com.intellij.execution.ExecutionException
import com.intellij.execution.process.CapturingProcessHandler
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.LocalFileSystem
import java.io.File

/** Competitive Companion's default port, which `cpl new --listen` waits on. */
const val COMPANION_PORT = 10043

/**
 * Import a problem from Competitive Companion, as VS Code's Import a Problem does: `cpl new --listen`
 * in the selected folder (else the project's), then open each file it creates.
 */
class CplImportProblemAction : AnAction() {
    override fun getActionUpdateThread() = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val picked = e.getData(CommonDataKeys.VIRTUAL_FILE)
        val folder = File((if (picked?.isDirectory == true) picked else picked?.parent)?.path ?: project.basePath ?: return)
        notify(project, "Waiting for Competitive Companion", "Open the problem in the browser and click the extension.", NotificationType.INFORMATION)
        object : Task.Backgroundable(project, "Waiting for Competitive Companion", true) {
            override fun run(indicator: ProgressIndicator) {
                val out = try {
                    CapturingProcessHandler(Cpl.commandLine(Cpl.companionArgs(COMPANION_PORT), folder))
                        .runProcessWithProgressIndicator(indicator)
                } catch (err: ExecutionException) {
                    notify(project, "No problem imported", err.message ?: "", NotificationType.ERROR)
                    return
                }
                if (out.isCancelled) return
                if (out.exitCode != 0) {
                    notify(project, "No problem imported", out.stderr.trim(), NotificationType.ERROR)
                    return
                }
                val files = Cpl.createdFiles(out.stdout).map { File(it).let { f -> if (f.isAbsolute) f else File(folder, it) } }
                ApplicationManager.getApplication().invokeLater {
                    for (file in files) {
                        val found = LocalFileSystem.getInstance().refreshAndFindFileByIoFile(file) ?: continue
                        FileEditorManager.getInstance(project).openFile(found, true)
                    }
                }
            }
        }.queue()
    }

    private fun notify(project: Project, title: String, text: String, type: NotificationType) =
        NotificationGroupManager.getInstance().getNotificationGroup("CP Lang").createNotification(title, text, type).notify(project)
}
