import React from "react";
import axios from "axios";
import { Oval } from "react-loader-spinner";
import { Buffer } from "buffer";
import init, {
  Crop,
  Options,
  OutputSize,
  Rotation,
  BuiltinBorder,
  Color,
  Sides,
  WasmImageBorders,
} from "filmborders";
import "./App.sass";
import hash from "object-hash";

type AppState = {
  wasmLoaded: boolean;
  workerReady: boolean;
  rendering: boolean;
  exporting: boolean;
  filename?: string;
  borderOverlay?: BuiltinBorder;
  borderOverlayName?: string;
  canvasScale: number;
  outputSizeName: string;
  outputWidth: number;
  outputHeight: number;
  backgroundColor: string;
  scaleFactor: number;
  cropTop?: number;
  cropRight?: number;
  cropBottom?: number;
  cropLeft?: number;
  borderWidthTop?: number;
  borderWidthRight?: number;
  borderWidthBottom?: number;
  borderWidthLeft?: number;
  rotationAngle?: Rotation;
  rotationAngleName?: string;
};

const PREVIEW_MAX_RES = 250;
const DEFAULT_BORDER_WIDTH = 10;
const OUTPUT_SIZES_KEYS: string[] = [
  "Insta Portrait",
  "Insta Landscape",
  "Insta Square",
  "Custom",
];
const OUTPUT_SIZES: { [key: string]: { width: number; height: number } } = {
  "Insta Portrait": {
    width: 1080,
    height: 1350,
  },
  "Insta Landscape": {
    width: 1080,
    height: 608,
  },
  "Insta Square": {
    width: 1080,
    height: 1080,
  },
};

const uuidv4 = (): string => {
  return "xxxxxxxx".replace(/[xy]/g, (c) => {
    let r = (Math.random() * 16) | 0,
      v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
};

type AppProps = {};

export default class App extends React.Component<AppProps, AppState> {
  protected originalCanvas = React.createRef<HTMLCanvasElement>();
  protected previewCanvas = React.createRef<HTMLCanvasElement>();
  protected borderCanvas = React.createRef<HTMLCanvasElement>();
  protected resultCanvas = React.createRef<HTMLCanvasElement>();

  protected canvas = React.createRef<HTMLCanvasElement>();
  protected canvasContainer = React.createRef<HTMLDivElement>();
  protected wasm!: typeof import("filmborders");
  protected worker!: Worker;
  protected resizeTimer?: ReturnType<typeof setTimeout>;
  protected updateTimer?: ReturnType<typeof setTimeout>;
  protected lastRenderConfigHash?: string;

  constructor(props: AppProps) {
    super(props);
    const outputSizeName = "Insta Portrait";
    const size = OUTPUT_SIZES[outputSizeName];
    this.state = {
      wasmLoaded: false,
      workerReady: false,
      rendering: false,
      exporting: false,
      filename: undefined,
      borderOverlay: undefined,
      borderOverlayName: undefined,
      canvasScale: 0.0,
      outputSizeName,
      outputWidth: size.width,
      outputHeight: size.height,
      backgroundColor: "#ffffff",
      scaleFactor: 0.95,
      cropTop: 0,
      cropRight: 0,
      cropBottom: 0,
      cropLeft: 0,
      borderWidthTop: DEFAULT_BORDER_WIDTH,
      borderWidthRight: DEFAULT_BORDER_WIDTH,
      borderWidthBottom: DEFAULT_BORDER_WIDTH,
      borderWidthLeft: DEFAULT_BORDER_WIDTH,
      rotationAngle: undefined,
      rotationAngleName: undefined,
    };
  }

  setWasmDefaults = async () => {
    const borderOverlay = BuiltinBorder.Border120_1;
    const rotationAngle = Rotation.Rotate0;

    await this.setState({
      rotationAngle,
      rotationAngleName: Rotation[rotationAngle],
      borderOverlay,
      borderOverlayName: BuiltinBorder[borderOverlay],
    });
  };

  init = async () => {
    if (this.state.wasmLoaded) return;
    try {
      await init();
      await this.setWasmDefaults();
      await this.setState({ wasmLoaded: true });
      console.log("loaded wasm");
    } catch (err: unknown) {
      console.error(`unexpected error when loading WASM: ${err}`);
      return;
    }
    try {
      await this.loadImage("/sample.jpg");
      await this.update(undefined, true);
      console.log("loaded image");
    } catch (err: unknown) {
      console.error(`unexpected error when loading image: ${err}`);
      return;
    }
  };

  getB64Image = async (url: string): Promise<string> => {
    try {
      let image = await axios.get(url, { responseType: "arraybuffer" });
      let raw = Buffer.from(image.data).toString("base64");
      return "data:" + image.headers["content-type"] + ";base64," + raw;
    } catch (error) {
      console.error(error);
      throw error;
    }
  };

  getOptions = async (): Promise<Options> => {
    let options = new Options();

    // output size
    let output_size = new OutputSize();
    output_size.width = this.state.outputWidth;
    output_size.height = this.state.outputHeight;
    options.output_size = output_size;

    // background_color
    try {
      options.background_color = new Color(this.state.backgroundColor);
    } catch (error) {
      console.error(error);
    }

    // scale factor
    options.scale_factor = this.state.scaleFactor ?? 1.0;

    // crop
    let crop = new Crop();
    crop.top = this.state.cropTop;
    crop.right = this.state.cropRight;
    crop.bottom = this.state.cropBottom;
    crop.left = this.state.cropLeft;
    options.crop = crop;

    // border width
    let borderWidth = new Sides();
    borderWidth.top = this.state.borderWidthTop ?? 0;
    borderWidth.right = this.state.borderWidthRight ?? 0;
    borderWidth.bottom = this.state.borderWidthBottom ?? 0;
    borderWidth.left = this.state.borderWidthLeft ?? 0;
    options.border_width = borderWidth;

    // rotation angle
    options.rotate_angle = this.state.rotationAngle;
    return options;
  };

  update = async (e?: React.FormEvent<HTMLFormElement>, force = false) => {
    if (!this.state.workerReady) return;
    try {
      console.log("render");

      e?.preventDefault();
      await this.resize();

      let config = {
        borderOverlay: this.state.borderOverlay,
        borderOverlayName: this.state.borderOverlayName,
        canvasScale: this.state.canvasScale,
        outputSizeName: this.state.outputSizeName,
        outputWidth: this.state.outputWidth,
        outputHeight: this.state.outputHeight,
        backgroundColor: this.state.backgroundColor,
        scaleFactor: this.state.scaleFactor,
        cropTop: this.state.cropTop,
        cropRight: this.state.cropRight,
        cropBottom: this.state.cropBottom,
        cropLeft: this.state.cropLeft,
        borderWidthTop: this.state.borderWidthTop,
        borderWidthRight: this.state.borderWidthTop,
        borderWidthBottom: this.state.borderWidthBottom,
        borderWidthLeft: this.state.borderWidthLeft,
        rotationAngleName: this.state.rotationAngleName,
        rotationAngle: this.state.rotationAngle,
      };

      let configHash = hash(config, { algorithm: "md5", encoding: "base64" });

      if (this.state.rendering) {
        console.log("skip render");
        if (force || configHash !== this.lastRenderConfigHash) {
          await this.scheduleUpdate(300);
        }
        return;
      }

      this.lastRenderConfigHash = configHash;
      const canvas = this.canvas.current;
      const previewCanvas = this.previewCanvas.current;
      const borderCanvas = this.borderCanvas.current;
      if (!canvas || !previewCanvas || !borderCanvas) return;

      const canvasCtx = canvas.getContext("2d");
      const previewCtx = previewCanvas.getContext("2d");
      const borderCtx = borderCanvas.getContext("2d");
      if (!canvasCtx || !previewCtx || !borderCtx) return;

      let renderID = uuidv4();
      console.time(renderID);
      await this.setState({ rendering: true });

      let imageData = previewCtx.getImageData(
        0,
        0,
        previewCanvas.width,
        previewCanvas.height
      );

      let borderData = null;
      if (this.state.borderOverlayName === "Custom") {
        borderData = borderCtx.getImageData(
          0,
          0,
          borderCanvas.width,
          borderCanvas.height
        );
      }

      await this.worker.postMessage({ imageData, borderData });
      let options = await this.getOptions();
      let size = new OutputSize();
      size.width = canvas.width;
      size.height = canvas.height;
      options.output_size = size;
      options.preview = true;
      await this.worker.postMessage({
        borderName: this.state.borderOverlay,
        applyOptions: options.serialize(),
        renderID,
        save: false,
      });
      console.log("waiting for worker");
    } catch (err) {
      await this.setState({ rendering: false });
      await this.scheduleUpdate();
      console.warn(err);
    }
  };

  exportHighRes = async (e?: React.MouseEvent<HTMLButtonElement>) => {
    await this.setState({ exporting: true });
    let originalCanvas = this.originalCanvas.current;
    let resultCanvas = this.resultCanvas.current;
    if (!resultCanvas || !originalCanvas) return;
    const originalCtx = originalCanvas.getContext("2d");
    const resultCtx = resultCanvas.getContext("2d");
    if (!resultCtx || !originalCtx) return;
    resultCanvas.width = this.state.outputWidth;
    resultCanvas.height = this.state.outputHeight;
    let renderID = uuidv4();
    console.time(renderID);
    let imgData = WasmImageBorders.to_image_data(originalCanvas, originalCtx);
    await this.worker.postMessage({ sourceImage: imgData });
    let options = await this.getOptions();
    options.preview = false;
    await this.worker.postMessage({
      applyOptions: options.serialize(),
      renderID,
      save: true,
    });
  };

  save = async (canvas: HTMLCanvasElement) => {
    let downloadLink = document.createElement("a");
    downloadLink.setAttribute(
      "download",
      this.state.filename ?? "exported.png"
    );
    let dataURL = canvas.toDataURL("image/png");
    let url = dataURL.replace(
      /^data:image\/png/,
      "data:application/octet-stream"
    );
    downloadLink.setAttribute("href", url);
    downloadLink.click();
  };

  drawToCanvas = (
    src: string,
    canvas: HTMLCanvasElement | null
  ): Promise<void> => {
    return new Promise((resolve, reject) => {
      if (!src) return reject();
      if (!canvas) return reject();
      let img: HTMLImageElement = new Image();
      img.onload = () => {
        let width = img.width;
        let height = img.height;
        canvas.width = width;
        canvas.height = height;
        canvas
          .getContext("2d")
          ?.drawImage(
            img,
            0,
            0,
            width,
            height,
            0,
            0,
            canvas.width,
            canvas.height
          );
        resolve();
      };
      img.src = src;
    });
  };

  loadBorderImage = async (src: string) => {
    await this.drawToCanvas(src, this.borderCanvas.current);
  };

  loadImage = (src: string): Promise<void> => {
    return new Promise((resolve, reject) => {
      let img: HTMLImageElement = new Image();
      img.onload = () => {
        let width = img.width;
        let height = img.height;

        let originalCanvas = this.originalCanvas.current;
        let previewCanvas = this.previewCanvas.current;
        let canvas = this.canvas.current;
        if (!previewCanvas || !originalCanvas || !canvas) return reject();
        let previewScaledownFac =
          PREVIEW_MAX_RES / Math.max(img.width, img.height);

        canvas.width = width * previewScaledownFac;
        canvas.height = height * previewScaledownFac;
        originalCanvas.width = width;
        originalCanvas.height = height;
        previewCanvas.width = canvas.width;
        previewCanvas.height = canvas.height;

        previewCanvas
          .getContext("2d")
          ?.drawImage(
            img,
            0,
            0,
            width,
            height,
            0,
            0,
            previewCanvas.width,
            previewCanvas.height
          );
        originalCanvas.getContext("2d")?.drawImage(img, 0, 0);
        return resolve();
      };
      img.src = src;
    });
  };

  renderToCanvas = (img: ImageData, canvas: HTMLCanvasElement | null) => {
    canvas?.getContext("2d")?.putImageData(img, 0, 0);
  };

  resize = async () => {
    let canvasContainer = this.canvasContainer.current;
    console.assert(canvasContainer);
    if (!canvasContainer) return;
    let canvasScale =
      Math.min(
        canvasContainer.clientWidth / this.state.outputWidth,
        canvasContainer.clientHeight / this.state.outputHeight
      ) * 0.95;
    await this.setState({ canvasScale });
    let canvas = this.canvas.current;
    console.assert(canvas);
    if (!canvas) return;
    let newWidth = Math.floor(this.state.outputWidth * canvasScale);
    let newHeight = Math.floor(this.state.outputHeight * canvasScale);
    // resizing causes the canvas to go blank
    if (canvas.width !== newWidth || canvas.height !== newHeight) {
      canvas.width = newWidth;
      canvas.height = newHeight;
    }
  };

  scheduleUpdate = async (timeout = 100) => {
    clearTimeout(this.updateTimer);
    this.updateTimer = setTimeout(this.update, timeout);
  };

  scheduleResize = async () => {
    clearTimeout(this.resizeTimer);
    this.resizeTimer = setTimeout(async () => {
      console.log("resize");
      await this.resize();
      await this.update(undefined, false);
    }, 300);
  };

  componentDidMount = async () => {
    this.worker = new Worker(
      `${process.env.PUBLIC_URL}/worker/ImageBorder.worker.js`
    );
    this.worker.onmessage = async (event) => {
      if ("status" in event.data) {
        if (event.data.status === "ready") {
          console.log("worker ready");
          await this.setState({ workerReady: true });
          await this.scheduleResize();
        }
      }
      if ("result" in event.data) {
        if (event.data.save) {
          let resultCanvas = this.resultCanvas.current;
          if (resultCanvas) {
            await this.renderToCanvas(event.data.result, resultCanvas);
            await this.save(resultCanvas);
            await this.update(undefined);
          }
          await this.setState({ exporting: false });
        } else {
          await this.renderToCanvas(event.data.result, this.canvas.current);
        }
        console.timeEnd(event.data.renderID);
        await this.setState({ rendering: false });
      }
    };

    await this.init();

    // let sampleImage = await this.getB64Image(
    //   "https://upload.wikimedia.org/wikipedia/commons/thumb/4/4c/Brad_Pitt_2019_by_Glenn_Francis.jpg/1200px-Brad_Pitt_2019_by_Glenn_Francis.jpg"
    // );

    window.addEventListener("resize", this.scheduleResize, false);
  };

  componentWillUnmount() {
    window.removeEventListener("resize", this.scheduleResize, false);
  }

  stripExtension = (filename: string): string => {
    return filename.replace(/\.[^/.]+$/, "");
  };

  openBorderImage = async (files: FileList | null) => {
    if (!files || files.length < 1) return;
    console.log(`loading ${files[0]}...`);
    let reader = new FileReader();
    reader.onload = async () => {
      await this.loadBorderImage(reader.result as string);
      await this.update(undefined, true);
    };
    reader.readAsDataURL(files[0]);
  };

  openImage = async (files: FileList | null) => {
    if (!files || files.length < 1) return;
    await this.setState({
      filename: `${this.stripExtension(files[0].name)}_border.png`,
    });
    console.log(`loading ${files[0]}...`);
    let reader = new FileReader();
    reader.onload = async () => {
      await this.loadImage(reader.result as string);
      await this.update(undefined, true);
    };
    reader.readAsDataURL(files[0]);
  };

  updateBorderOverlay = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    let key = e.target.value as keyof typeof BuiltinBorder;
    if (key in BuiltinBorder) {
      await this.setState({
        borderOverlay: BuiltinBorder[key],
        borderOverlayName: BuiltinBorder[BuiltinBorder[key]],
      });
    } else {
      await this.setState({
        borderOverlayName: key,
      });
    }
    await this.scheduleUpdate();
  };

  updateRotationAngle = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    let key = e.target.value as keyof typeof Rotation;
    await this.setState({
      rotationAngle: Rotation[key],
      rotationAngleName: Rotation[Rotation[key]],
    });
    await this.scheduleUpdate();
  };

  updateOutputSize = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    let key = e.target.value as keyof typeof OUTPUT_SIZES;
    if (key in OUTPUT_SIZES) {
      await this.setState({
        outputSizeName: e.target.value,
        outputWidth: OUTPUT_SIZES[key].width,
        outputHeight: OUTPUT_SIZES[key].height,
      });
    } else {
      await this.setState({
        outputSizeName: e.target.value,
      });
    }
    await this.scheduleUpdate();
  };

  updateOutputWidth = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ outputWidth: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };

  updateOutputHeight = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ outputHeight: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };

  updateBackgroundColor = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ backgroundColor: e.target.value });
    await this.scheduleUpdate();
  };

  updateScaleFactor = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ scaleFactor: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };

  updateBorderWidth = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({
      borderWidthTop: parseFloat(e.target.value),
      borderWidthRight: parseFloat(e.target.value),
      borderWidthBottom: parseFloat(e.target.value),
      borderWidthLeft: parseFloat(e.target.value),
    });
    await this.scheduleUpdate();
  };

  updateCropTop = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ cropTop: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };
  updateCropRight = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ cropRight: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };
  updateCropBottom = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ cropBottom: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };
  updateCropLeft = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ cropLeft: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };

  render = () => {
    return (
      <div id="app">
        <canvas className="offscreen" ref={this.resultCanvas}></canvas>
        <canvas className="offscreen" ref={this.previewCanvas}></canvas>
        <canvas className="offscreen" ref={this.originalCanvas}></canvas>
        <canvas className="offscreen" ref={this.borderCanvas}></canvas>

        <div id="wasm-canvas-container" ref={this.canvasContainer}>
          <Oval
            wrapperClass={
              "spinner " +
              (this.state.rendering ||
              this.state.exporting ||
              !this.state.wasmLoaded ||
              !this.state.workerReady
                ? "visible"
                : "")
            }
            color="#80cbc4"
            secondaryColor="#e1f5fe"
          />
          <canvas id="wasm-canvas" ref={this.canvas}></canvas>
        </div>

        <form className="parameters" onSubmit={this.update}>
          <fieldset>
            <div className="formgrid">
              <input
                type="file"
                disabled={this.state.rendering || this.state.exporting}
                onChange={(e) => this.openImage(e.target.files)}
                name="imageInput"
                id="imageInput"
                accept="image/*"
              />
              <label htmlFor="imageInput">Photo</label>

              <select
                id="borderOverlay"
                value={this.state.borderOverlayName}
                disabled={this.state.exporting}
                onChange={this.updateBorderOverlay}
              >
                {Object.values(BuiltinBorder)
                  .filter((r) => typeof r == "string")
                  .map((option) => (
                    <option value={option} key={option}>
                      {option}
                    </option>
                  ))}
                <option value="Custom" key="Custom">
                  Custom
                </option>
              </select>
              <label htmlFor="borderOverlay">Border</label>

              <input
                type="file"
                disabled={
                  this.state.borderOverlayName !== "Custom" ||
                  this.state.rendering ||
                  this.state.exporting
                }
                onChange={(e) => this.openBorderImage(e.target.files)}
                name="borderInput"
                id="borderInput"
                accept="image/*"
              />
              <label htmlFor="borderInput"></label>

              <select
                id="outputSize"
                disabled={this.state.exporting}
                value={this.state.outputSizeName}
                onChange={this.updateOutputSize}
              >
                {OUTPUT_SIZES_KEYS.map((size) => (
                  <option value={size} key={size}>
                    {size}
                  </option>
                ))}
              </select>
              <label htmlFor="outputSize">Size</label>

              <input
                id="outputWidth"
                type="number"
                step="1"
                disabled={
                  this.state.exporting || this.state.outputSizeName !== "Custom"
                }
                value={this.state.outputWidth}
                onChange={this.updateOutputWidth}
              />
              <label htmlFor="outputWidth">Width</label>

              <input
                id="outputHeight"
                type="number"
                step="1"
                disabled={
                  this.state.exporting || this.state.outputSizeName !== "Custom"
                }
                value={this.state.outputHeight}
                onChange={this.updateOutputHeight}
              />
              <label htmlFor="outputHeight">Height</label>

              <input
                id="backgroundColor"
                type="color"
                disabled={this.state.exporting}
                value={this.state.backgroundColor}
                onChange={this.updateBackgroundColor}
              />
              <label htmlFor="backgroundColor">Color</label>

              <select
                id="rotationAngle"
                value={this.state.rotationAngleName}
                disabled={this.state.exporting}
                onChange={this.updateRotationAngle}
              >
                {Object.values(Rotation)
                  .filter((r) => typeof r == "string")
                  .map((option) => (
                    <option value={option} key={option}>
                      {option}
                    </option>
                  ))}
              </select>
              <label htmlFor="rotationAngle">Rotation</label>

              <input
                id="scaleFactor"
                type="number"
                step="0.01"
                disabled={this.state.exporting}
                value={this.state.scaleFactor}
                onChange={this.updateScaleFactor}
              />
              <label htmlFor="scaleFactor">Scale factor</label>

              <input
                type="number"
                id="borderWidth"
                disabled={this.state.exporting}
                value={this.state.borderWidthTop}
                onChange={this.updateBorderWidth}
              />
              <label htmlFor="borderWidth">Border width</label>

              <input
                type="number"
                id="cropTop"
                disabled={this.state.exporting}
                value={this.state.cropTop}
                onChange={this.updateCropTop}
              />
              <label htmlFor="cropTop">Crop top</label>

              <input
                type="number"
                id="cropRight"
                disabled={this.state.exporting}
                value={this.state.cropRight}
                onChange={this.updateCropRight}
              />
              <label htmlFor="cropRight">Crop right</label>

              <input
                type="number"
                id="cropBottom"
                disabled={this.state.exporting}
                value={this.state.cropBottom}
                onChange={this.updateCropBottom}
              />
              <label htmlFor="cropBottom">Crop bottom</label>

              <input
                type="number"
                id="cropLeft"
                disabled={this.state.exporting}
                value={this.state.cropLeft}
                onChange={this.updateCropLeft}
              />
              <label htmlFor="cropLeft">Crop left</label>

              <button
                disabled={this.state.rendering || this.state.exporting}
                type="submit"
              >
                Update
              </button>
              <button
                disabled={this.state.rendering || this.state.exporting}
                onClick={this.exportHighRes}
              >
                Export
              </button>
            </div>
            <div className="about">
              <p>WASM based film border overlays.</p>
              <p>
                by <a href="https://romnn.com">@romnn</a>
              </p>
              <p>
                code on{" "}
                <a href="https://github.com/romnn/film-borders">github</a>
              </p>
            </div>
          </fieldset>
        </form>
      </div>
    );
  };
}
