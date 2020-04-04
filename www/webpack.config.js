const CopyPlugin = require('copy-webpack-plugin');
const path = require('path');

module.exports = {
  entry: './src/bootstrap.js',
  output: {
    filename: 'bootstrap.js',
    path: path.resolve(__dirname, 'dist'),
  },
  plugins: [
    new CopyPlugin([
      { from: 'src', ignore: [ '.DS_Store' ] },
    ]),
  ],
  devServer: {
    contentBase: "./src",
    host: '0.0.0.0',
    port: 8080,
    disableHostCheck: true,
  },
};
