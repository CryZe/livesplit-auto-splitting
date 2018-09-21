const path = require('path');

module.exports = {
	mode: 'development',
	entry: {
		"app": './index.js',
		"editor.worker": 'monaco-editor/esm/vs/editor/editor.worker.js',
	},
	devServer: {
		contentBase: './dist'
	},
	output: {
		globalObject: 'self',
		filename: '[name].bundle.js',
		path: path.resolve(__dirname, 'dist')
	},
	module: {
		rules: [{
			test: /\.css$/,
			use: ['style-loader', 'css-loader']
		}]
	},
	node: {
		fs: "empty"
	},
};
