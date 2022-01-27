let { setNative } = require("./common.cjs")
let os = require("os")

module.exports = require("./common.cjs")

if (!globalThis.fetch) {
	globalThis.fetch = require("node-fetch")
}
let target = null
let arch = os.arch()
let platform = os.platform()
if (arch === "x64" && platform === "linux") {
	target = "x86_64-unknown-linux-gnu"
} else if (arch === "arm64" && platform === "linux") {
	target = "aarch64-unknown-linux-gnu"
} else if (arch === "x64" && platform === "darwin") {
	target = "x86_64-apple-darwin"
} else if (arch === "arm64" && platform === "darwin") {
	target = "aarch64-apple-darwin"
} else if (arch === "x64" && platform === "win32") {
	target = "x86_64-pc-windows-msvc"
}

if (target !== null) {
	setNative(require(`./tangram_${target}.node`))
} else {
	setNative(require("./tangram_wasm.cjs"))
}
