// Resolves the platform-specific Weavatrix SEO binary bundled in the universal
// npm package. Pure Node: built-ins only, no install scripts and no network.
import { existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const BUNDLED_BINARIES = {
    'win32 x64': ['win32-x64', 'weavatrix-seo.exe'],
    'win32 arm64': ['win32-arm64', 'weavatrix-seo.exe'],
    'darwin x64': ['darwin-x64', 'weavatrix-seo'],
    'darwin arm64': ['darwin-arm64', 'weavatrix-seo'],
    'linux x64': ['linux-x64', 'weavatrix-seo'],
    'linux arm64': ['linux-arm64', 'weavatrix-seo'],
}

export function resolveBinary() {
    const key = `${process.platform} ${process.arch}`
    const entry = BUNDLED_BINARIES[key]
    if (!entry) {
        fail(
            `Unsupported platform: ${key}.`,
            'Prebuilt binaries cover win32/darwin/linux on x64 and arm64.',
            'On other platforms build https://github.com/Weavatrix/weavatrix-seo from source.',
        )
    }
    const [directory, binaryName] = entry
    const binary = join(dirname(fileURLToPath(import.meta.url)), 'native', directory, binaryName)
    if (!existsSync(binary)) {
        fail(
            `The bundled native executable for ${key} is missing.`,
            'The installed package is incomplete; reinstall weavatrix-seo,',
            'or build https://github.com/Weavatrix/weavatrix-seo from source.',
        )
    }
    return binary
}

function fail(...lines) {
    for (const line of lines) console.error(`weavatrix-seo: ${line}`)
    process.exit(1)
}
