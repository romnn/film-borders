const href = location.href;
const workerPath = '/worker/ImageBorder.worker.js';
const baseURL = href.replace(workerPath, '');

importScripts(`${baseURL}/wasm/filmborders.js`);

const init_wasm_in_worker = async () => {
  await wasm_bindgen(`${baseURL}/wasm/filmborders_bg.wasm`);
  const {ImageBorders, Builtin, Border, Options} = wasm_bindgen;

  let imageData = null;   // ImageData
  let borderData = null;  // ImageData

  self.postMessage({status: 'ready'});

  self.onmessage = async (event) => {
    let message = event.data;

    console.log(`worker: received message: ${message}`);
    if ('imageData' in message) {
      imageData = message.imageData;
      borderData = message.borderData;
    }
    if ('options' in message) {
      let {options, renderID, save, borderName} = message;
      let image = ImageBorders.from_image_data(imageData);
      let border = new Border(borderData, borderName);
      try {
      let result = image.add_border(border, Options.deserialize(options));
      self.postMessage({result, renderID, save});
      console.log(`worker: render ${renderID} done`);
      } catch (err) {
        console.error("error in worker", err);
      }
    }
  };
};

init_wasm_in_worker();
