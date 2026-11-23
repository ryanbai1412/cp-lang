package dev.cplang.ide

import com.intellij.ide.impl.OpenProjectTask
import com.intellij.ide.impl.ProjectUtil
import com.intellij.ide.lightEdit.LightEdit
import com.intellij.ide.lightEdit.LightEditCompatible
import com.intellij.openapi.fileEditor.FileEditor
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.ui.EditorNotificationPanel
import com.intellij.ui.EditorNotificationProvider
import java.util.function.Function
import javax.swing.JComponent

/**
 * LightEdit (a file opened on its own, outside a project) has no project for
 * a language server, so a .cp file there gets highlighting only. Say so, and
 * offer to open its folder as a project, where `cpl lsp` runs.
 */
class CpLightEditNotice : EditorNotificationProvider, LightEditCompatible {
    override fun collectNotificationData(project: Project, file: VirtualFile): Function<in FileEditor, out JComponent?>? {
        if (file.extension != "cp" || !LightEdit.owns(project)) return null
        val folder = file.parent ?: return null
        return Function { editor ->
            EditorNotificationPanel(editor, EditorNotificationPanel.Status.Warning).apply {
                text = MESSAGE
                createActionLabel("Open ${folder.name} as a project") {
                    ProjectUtil.openOrImport(folder.toNioPath(), OpenProjectTask { })
                }
            }
        }
    }

    companion object {
        const val MESSAGE =
            "LightEdit mode has no project, so CP Lang only highlights this file: no diagnostics, completion or runs."
    }
}
