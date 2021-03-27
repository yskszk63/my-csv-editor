const path = require("path");
const autoprefixer = require('autoprefixer');
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  experiments: {
    asyncWebAssembly: true,
  },
  entry: {
    index: ["./js/index.js", "./css/app.scss"],
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    contentBase: dist,
  },
  module: {
    rules: [
      {
        test: /\.scss$/,
        use: [
          {
            loader: "file-loader",
            options: {
              name: "bundle.css",
            },
          },
          {
            loader: "extract-loader",
          },
          {
            loader: "css-loader",
          },
          {
            loader: "postcss-loader",
            options: {
              plugins: () => [autoprefixer()],
            },
          },
          {
            loader: "sass-loader",
            options: {
              implementation: require("sass"),
              webpackImporter: false,
              sassOptions: {
                includePaths: ["./node_modules"],
              },
            },
          },
        ]
      }
    ],
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        { from: path.resolve(__dirname, "static") }
      ]
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ]
};
