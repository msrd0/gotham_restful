import * as path from 'path';
import * as webpack from 'webpack';

const cfg: webpack.Configuration = {
	entry: './script.ts',
	module: {
		rules: [
			{
				test: /\.ts/,
				use: 'ts-loader',
				exclude: /node_modules/
			}
		]
	},
	resolve: {
		extensions: ['.ts', '.js']
	},
	output: {
		filename: 'script.min.js',
		path: path.resolve(__dirname),
		library: {
			name: 'gotham_restful',
			type: 'var',
			export: 'default'
		}
	}
};

export default cfg;
