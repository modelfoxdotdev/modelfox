import { setNative } from "./common.js"

export * from "./common.js"

setNative(await import("./modelfox_wasm.js"))
