import { setNative } from "./model.js"

export * from "./model.js"

setNative(await import("../wasm/dist/bundler/tangram_wasm.js"))
