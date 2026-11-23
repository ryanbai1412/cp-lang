import org.jetbrains.intellij.platform.gradle.IntelliJPlatformType
import org.jetbrains.intellij.platform.gradle.TestFrameworkType
import org.jetbrains.intellij.platform.gradle.tasks.VerifyPluginTask

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.0.21"
    id("org.jetbrains.intellij.platform") version "2.2.1"
}

group = "dev.cplang"
// The plugin's version is the workspace's, so the zip says what it ships with.
version = Regex("""(?ms)^\[workspace\.package\][^\[]*?^version = "([^"]+)"""")
    .find(rootDir.resolve("../../Cargo.toml").readText())!!.groupValues[1]

kotlin {
    jvmToolchain(21)
}

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        // The built-in LSP client ships only in the commercial IDEs (IntelliJ IDEA
        // Ultimate, CLion, PyCharm Professional, ...), so the plugin builds against
        // one; Community IDEs get `cpl lsp` through LSP4IJ. Both are optional.
        intellijIdeaUltimate(providers.gradleProperty("platformVersion"))
        plugin("com.redhat.devtools.lsp4ij", providers.gradleProperty("lsp4ijVersion").get())
        testFramework(TestFrameworkType.Platform)
        pluginVerifier()
    }
    testImplementation("junit:junit:4.13.2")
}

intellijPlatform {
    pluginConfiguration {
        version = project.version.toString()
        ideaVersion {
            sinceBuild = "243"
            untilBuild = provider { null }
        }
    }
    // `./gradlew verifyPlugin`: the built zip against IntelliJ IDEA Community, where
    // the built-in LSP client is missing, and Ultimate, where it is used. A required
    // dependency Community lacks is a compatibility problem and fails it; the optional
    // `com.intellij.modules.ultimate` is reported missing there, as it should be.
    pluginVerification {
        ides {
            ide(IntelliJPlatformType.IntellijIdeaCommunity, providers.gradleProperty("platformVersion").get())
            ide(IntelliJPlatformType.IntellijIdeaUltimate, providers.gradleProperty("platformVersion").get())
        }
        failureLevel = listOf(
            VerifyPluginTask.FailureLevel.COMPATIBILITY_PROBLEMS,
            VerifyPluginTask.FailureLevel.INVALID_PLUGIN,
        )
    }
    buildSearchableOptions = false
    // No GUI forms or @NotNull to instrument, and its compiler is one more download.
    instrumentCode = false
}

val repo: File = rootDir.parentFile.parentFile

tasks {
    test {
        systemProperty("cpl.repo", repo.absolutePath)
        systemProperty("cpl.bin", System.getenv("CPL") ?: repo.resolve(if (System.getProperty("os.name").startsWith("Windows")) "target/debug/cpl.exe" else "target/debug/cpl").absolutePath)
    }
}
