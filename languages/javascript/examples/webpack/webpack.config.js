module.exports = {
	entry: "./index.js",
	experiments: {
		topLevelAwait: true,
		asyncWebAssembly: true,
	},
	module: {
		rules: [
			{
				test: /\.tangram$/,
				use: "file-loader",
			},
		],
	},
}
