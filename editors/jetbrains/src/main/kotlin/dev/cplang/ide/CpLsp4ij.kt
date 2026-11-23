package dev.cplang.ide

import com.intellij.openapi.project.Project
import com.redhat.devtools.lsp4ij.LanguageServerEnablementSupport
import com.redhat.devtools.lsp4ij.LanguageServerFactory
import com.redhat.devtools.lsp4ij.server.OSProcessStreamConnectionProvider
import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider
import java.io.File

/**
 * `cpl lsp` through LSP4IJ, for the IDEs without the built-in LSP client (IntelliJ IDEA
 * Community, PyCharm Community, Android Studio). Off where the built-in client runs it.
 */
class CpLsp4ijFactory : LanguageServerFactory, LanguageServerEnablementSupport {
    override fun createConnectionProvider(project: Project): StreamConnectionProvider =
        OSProcessStreamConnectionProvider(
            Cpl.commandLine(Cpl.lspArgs(), project.basePath?.let(::File)?.takeIf { it.isDirectory }),
        )

    override fun isEnabled(project: Project) = !builtInClient

    override fun setEnabled(enabled: Boolean, project: Project) {}

    companion object {
        val builtInClient: Boolean =
            runCatching { Class.forName("com.intellij.platform.lsp.api.LspServerSupportProvider") }.isSuccess
    }
}
