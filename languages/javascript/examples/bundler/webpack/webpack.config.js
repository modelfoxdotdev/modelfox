module.exports = {
	mode: "production",
	entry: "./index.js",
	experiments: {
		asyncWebAssembly: true,
		topLevelAwait: true,
	},
	module: {
		rules: [
			{
				test: /\.tangram$/,
				type: "asset/resource",
			},
		],
	},
}
