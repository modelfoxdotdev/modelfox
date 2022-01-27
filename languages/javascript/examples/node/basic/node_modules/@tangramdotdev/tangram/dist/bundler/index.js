import { setNative } from "./common.js"

export * from "./common.js"

setNative(await import("./tangram_wasm.js"))
