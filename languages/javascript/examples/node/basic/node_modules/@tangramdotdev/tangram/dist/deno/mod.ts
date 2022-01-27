import { setNative } from "./common.ts"

export * from "./common.ts"

setNative(await import("./tangram_wasm.js"))
