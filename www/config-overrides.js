const path = require('path');

module.exports = function override(config, env) {
  const wasmExtensionRegExp = /\.wasm$/;
  // const workerExtensionRegExp = /\.worker\.js$/;

  config.resolve.extensions.push('.wasm');

  config.module.rules.forEach(rule => {
    (rule.oneOf || []).forEach(oneOf => {
      if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
        // make file-loader ignore WASM and worker files
        oneOf.exclude.push(wasmExtensionRegExp);
        // oneOf.exclude.push(workerExtensionRegExp);
      }
    });
  });

  // add a dedicated loader for WASM
  config.module.rules.push({
    test : wasmExtensionRegExp,
    include : path.resolve(__dirname, 'src'),
    use : [ {loader : require.resolve('wasm-loader'), options : {}} ]
  });

  // add a dedicated loader for web workers
  // config.module.rules.unshift({
  // config.module.rules.push({
  //   test : workerExtensionRegExp,
  //   include : path.resolve(__dirname, 'src'),
  //   use : {
  //     loader : 'worker-loader',
  //     options : {
  //       filename : 'static/js/[name].[contenthash:8].js',
  //     },
  //   },
  // });

  return config;
};
