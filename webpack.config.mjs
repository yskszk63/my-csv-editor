import { fileURLToPath } from "node:url";

import sass from "sass";
import CopyPlugin from "copy-webpack-plugin";
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";

const dist = fileURLToPath(import.meta.resolve("./dist"));

const config = {
  mode: "production",
  experiments: {
    asyncWebAssembly: true,
  },
  entry: {
    index: ["./js/index.js"],
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    static: {
      directory: dist,
    },
  },
  module: {
    rules: [
      {
        test: /\.scss$/,
        use: [
          {
            loader: "style-loader",
          },
          {
            loader: "css-loader",
          },
          {
            loader: "postcss-loader",
            options: {
              postcssOptions: {
                plugins: [
                  [
                    "autoprefixer",
                    { },
                  ]
                ]
              }
            },
          },
          {
            loader: "sass-loader",
            options: {
              implementation: sass,
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
        { from: fileURLToPath(import.meta.resolve("./static")) }
      ]
    }),
    new WasmPackPlugin({
      crateDirectory: import.meta.dirname,
    }),
  ],
};

export default config;
