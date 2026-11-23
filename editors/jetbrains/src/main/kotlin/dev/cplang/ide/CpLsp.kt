package dev.cplang.ide

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import java.io.File

/**
 * Diagnostics, hover, completion, navigation, rename, quick fixes and inlay hints all come from
 * `cpl lsp`, through the commercial IDEs' built-in client when LSP4IJ isn't loaded.
 */
class CpLspServerSupportProvider : LspServerSupportProvider {
    override fun fileOpened(
        project: Project,
        file: VirtualFile,
        serverStarter: LspServerSupportProvider.LspServerStarter,
    ) {
        if (file.extension == "cp" && !CpLsp4ijFactory.loaded) {
            serverStarter.ensureServerStarted(CpLspServerDescriptor(project))
        }
    }
}

class CpLspServerDescriptor(project: Project) : ProjectWideLspServerDescriptor(project, "CP Lang") {
    override fun isSupportedFile(file: VirtualFile) = file.extension == "cp"

    override fun createCommandLine(): GeneralCommandLine =
        Cpl.commandLine(Cpl.lspArgs(), project.basePath?.let(::File)?.takeIf { it.isDirectory })
}
