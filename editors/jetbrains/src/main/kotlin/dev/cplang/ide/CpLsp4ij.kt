package dev.cplang.ide

import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.project.Project
import com.redhat.devtools.lsp4ij.LanguageServerFactory
import com.redhat.devtools.lsp4ij.server.OSProcessStreamConnectionProvider
import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider
import java.io.File

/**
 * `cpl lsp` through LSP4IJ, wherever LSP4IJ is loaded: `cpl setup jetbrains` installs it
 * in every IDE, because the free IntelliJ IDEA ships the built-in client's classes without
 * the module that turns it on. The built-in client runs `cpl lsp` only where LSP4IJ is absent.
 */
class CpLsp4ijFactory : LanguageServerFactory {
    override fun createConnectionProvider(project: Project): StreamConnectionProvider =
        OSProcessStreamConnectionProvider(
            Cpl.commandLine(Cpl.lspArgs(), project.basePath?.let(::File)?.takeIf { it.isDirectory }),
        )

    companion object {
        private val id = PluginId.getId("com.redhat.devtools.lsp4ij")

        val loaded: Boolean
            get() = PluginManagerCore.getPlugin(id)?.let { !PluginManagerCore.isDisabled(id) } ?: false
    }
}
