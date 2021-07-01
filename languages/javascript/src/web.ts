import init from "../wasm/dist/web/tangram_wasm.js"
import { setNative } from "./model.js"
export * from "./model.js"

setNative(await init())
