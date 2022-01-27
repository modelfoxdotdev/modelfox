import { setNative } from "./common.js"
import init, * as native from "./tangram_wasm.js"

export * from "./common.js"

await init()
setNative(native)
