package dev.cplang.ide

import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.project.Project
import com.redhat.devtools.lsp4ij.LanguageServerFactory
import com.redhat.devtools.lsp4ij.commands.LSPCommand
import com.redhat.devtools.lsp4ij.commands.LSPCommandAction
import com.redhat.devtools.lsp4ij.server.OSProcessStreamConnectionProvider
import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider
import com.intellij.openapi.vfs.VirtualFile
import java.io.File

/**
 * `cpl lsp` through LSP4IJ, wherever LSP4IJ is loaded: `cpl setup jetbrains` installs it
 * in every IDE, because the free IntelliJ IDEA ships the built-in client's classes without
 * the module that turns it on. The built-in client runs `cpl lsp` only where LSP4IJ is absent.
 */
class CpLsp4ijFactory : LanguageServerFactory {
    override fun createConnectionProvider(project: Project): StreamConnectionProvider =
        object : OSProcessStreamConnectionProvider(
            Cpl.commandLine(Cpl.lspArgs(), project.basePath?.let(::File)?.takeIf { it.isDirectory }),
        ) {
            override fun getInitializationOptions(rootUri: VirtualFile?): Any = INITIALIZATION_OPTIONS
        }

    companion object {
        /** Asks `cpl lsp` for its Run, Trace, Test and Stress code lenses. */
        val INITIALIZATION_OPTIONS = mapOf("codeLens" to true)

        private val id = PluginId.getId("com.redhat.devtools.lsp4ij")

        val loaded: Boolean
            get() = PluginManagerCore.getPlugin(id)?.let { !PluginManagerCore.isDisabled(id) } ?: false
    }
}

/** `cpl.exec`, the command of `cpl lsp`'s code lenses: LSP4IJ runs the action of that id. */
class CplExecAction : LSPCommandAction() {
    override fun getCommandPerformedThread() = ActionUpdateThread.EDT

    override fun commandPerformed(command: LSPCommand, e: AnActionEvent) {
        val project = e.project ?: return
        CplExec.run(project, command.arguments.indices.mapNotNull { command.getArgumentAt<String>(it, String::class.java) })
    }
}
