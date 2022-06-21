const href = location.href;
const workerPath = "/worker/ImageBorder.worker.js";
const baseURL = href.replace(workerPath, '');

importScripts(`${baseURL}/wasm/filmborders.js`)

const init_wasm_in_worker = async () => {
  wasm_bindgen(`${baseURL}/wasm/filmborders_bg.wasm`).then(wasm => {
    const {ImageBorders, BorderOptions} = wasm_bindgen;
    let sourceImage = null;
    self.postMessage({status : "ready"});
    self.onmessage = async (event) => {
      // console.log("message in worker", event);
      if ("sourceImage" in event.data) {
        console.log("setting source image", event.data.sourceImage);
        sourceImage = event.data.sourceImage;
      };
      if ("applyOptions" in event.data) {
        let renderID = event.data.renderID;
        let save = event.data.save;
        console.log(renderID, "applying", JSON.parse(event.data.applyOptions),
                    "to", sourceImage);
        let wasmImg = ImageBorders.for_image_data(sourceImage);
        let options = BorderOptions.deserialize(event.data.applyOptions);
        let result = wasmImg.apply_wasm(options);
        self.postMessage({result, renderID, save});
      };
    };
  });
};

init_wasm_in_worker();
