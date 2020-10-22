const CopyPlugin = require('copy-webpack-plugin');
const path = require('path');

module.exports = {
  entry: './www/bootstrap.js',
  output: {
    filename: 'bootstrap.js',
    path: path.resolve(__dirname, 'docs'),
  },
  plugins: [
    new CopyPlugin([
      { from: 'www/', to: '' },
      { from: 'examples/', to: 'examples/' },
    ]),
  ],
  devServer: {
    contentBase: "./www",
    host: '0.0.0.0',
    port: 8080,
    disableHostCheck: true,
    headers: {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, PATCH, OPTIONS",
      "Access-Control-Allow-Headers": "X-Requested-With, content-type, Authorization"
    },
  },
};
