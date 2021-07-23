const href = location.href;
const workerPath = location.pathname;
const baseURL = href.replace(workerPath, '');

importScripts(`${baseURL}/wasm/wasm_film_borders.js`)

const init_wasm_in_worker = async () => {
  wasm_bindgen(`${baseURL}/wasm/wasm_film_borders_bg.wasm`).then(wasm => {
    const {WasmImageBorders, ImageBorderOptions} = wasm_bindgen;
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
        let wasmImg = WasmImageBorders.for_image_data(sourceImage);
        let options = ImageBorderOptions.deserialize(event.data.applyOptions);
        let result = wasmImg.apply(options);
        self.postMessage({result, renderID, save});
      };
    };
  });
};

init_wasm_in_worker();
