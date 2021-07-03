import { setNative } from "./model.js"

export * from "./model.js"

setNative(await import("../wasm/dist/deno/tangram_wasm.js"))
