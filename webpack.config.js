const CopyPlugin = require('copy-webpack-plugin');
const path = require('path');

module.exports = {
  entry: './www/bootstrap.js',
  output: {
    filename: 'bootstrap.js',
    path: path.resolve(__dirname, 'dist'),
  },
  plugins: [
    new CopyPlugin([
      { from: 'www', ignore: [ '.DS_Store' ] },
    ]),
  ],
  devServer: {
    contentBase: "./www",
    host: '0.0.0.0',
    port: 8080,
    disableHostCheck: true,
  },
};
