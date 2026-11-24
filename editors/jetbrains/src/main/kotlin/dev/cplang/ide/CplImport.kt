package dev.cplang.ide

import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.redhat.devtools.lsp4ij.LanguageServerManager
import org.eclipse.lsp4j.ExecuteCommandParams

/**
 * Import a Problem: `cpl lsp`'s `cpl.import` waits for Competitive Companion, creates each problem
 * in the selected folder (else the project's) and opens it through `window/showDocument`.
 */
class CplImportProblemAction : AnAction() {
    override fun getActionUpdateThread() = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val path = e.getData(CommonDataKeys.VIRTUAL_FILE)?.path ?: project.basePath
        LanguageServerManager.getInstance(project).getLanguageServer("cpl").thenAccept {
            it?.server?.workspaceService?.executeCommand(ExecuteCommandParams("cpl.import", listOf(path)))
        }
    }
}
