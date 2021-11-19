const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: "index.html"
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, ".")
        }),
        new webpack.ProvidePlugin({
            TextDecoder: ['text-encoding', 'TextDecoder'],
            TextEncoder: ['text-encoding', 'TextEncoder']
        }),
        new CopyWebpackPlugin({
            patterns: [
                { from: "welkin", to: "" },
                { from: "assets", to: "" }
            ],
        }),
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
    },
    devServer: {
        static: [{
                directory: path.join(__dirname, 'welkin')
            },
            {
                directory: path.join(__dirname, 'assets')
            }
        ]
    },
    ignoreWarnings: [
        (warning) =>
        warning.message ===
        "Critical dependency: the request of a dependency is an expression",
    ],
};