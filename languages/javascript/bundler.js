import { setNative } from "./common.js"

export * from "./common.js"

setNative(await import("./wasm/dist/bundler/tangram_wasm.js"))
