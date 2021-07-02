export default {
	mode: "production",
	entry: "./dist/bundler.js",
	output: {
		filename: "./index.js",
		library: {
			type: "module",
		},
	},
	experiments: {
		asyncWebAssembly: true,
		outputModule: true,
		topLevelAwait: true,
	},
}
