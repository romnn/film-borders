import React from "react";
import axios from "axios";
import { Oval } from "react-loader-spinner";
import { Buffer } from "buffer";
import init, {
  Crop,
  BorderOptions,
  OutputSize,
  Rotation,
  Sides,
  ImageBorders,
} from "filmborders";
import "./App.sass";
import "react-loader-spinner/dist/loader/css/react-spinner-loader.css";
import hash from "object-hash";

type AppState = {
  wasmLoaded: boolean;
  workerReady: boolean;
  rendering: boolean;
  filename?: string;
  canvasScale: number;
  outputSizeName: string;
  outputWidth: number;
  outputHeight: number;
  scaleFactor: number;
  cropTop?: number;
  cropRight?: number;
  cropBottom?: number;
  cropLeft?: number;
  borderWidthTop?: number;
  borderWidthRight?: number;
  borderWidthBottom?: number;
  borderWidthLeft?: number;
  rotationAngleName?: string;
  rotationAngle?: number;
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

type AppProps = {};

const uuidv4 = (): string => {
  return "xxxxxxxx".replace(/[xy]/g, (c) => {
    let r = (Math.random() * 16) | 0,
      v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
};

export default class App extends React.Component<AppProps, AppState> {
  protected originalSrcCanvas = React.createRef<HTMLCanvasElement>();
  protected previewSrcCanvas = React.createRef<HTMLCanvasElement>();
  protected resultCanvas = React.createRef<HTMLCanvasElement>();
  protected canvas = React.createRef<HTMLCanvasElement>();
  protected canvasContainer = React.createRef<HTMLDivElement>();
  protected wasm!: typeof import("filmborders");
  protected worker!: Worker;
  protected img!: HTMLImageElement;
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
      filename: undefined,
      canvasScale: 0.0,
      outputSizeName,
      outputWidth: size.width,
      outputHeight: size.height,
      scaleFactor: 0.95,
      cropTop: 0,
      cropRight: 0,
      cropBottom: 0,
      cropLeft: 0,
      borderWidthTop: DEFAULT_BORDER_WIDTH,
      borderWidthRight: DEFAULT_BORDER_WIDTH,
      borderWidthBottom: DEFAULT_BORDER_WIDTH,
      borderWidthLeft: DEFAULT_BORDER_WIDTH,
      rotationAngleName: undefined,
      rotationAngle: undefined,
    };
  }

  loadWasm = async (): Promise<void> => {
    if (this.state.wasmLoaded) return;
    try {
      await init();
      this.setState({ wasmLoaded: true });
    } catch (err: unknown) {
      console.error(`unexpected error when loading WASM. (${err})`);
    }
  };

  getB64Image = async (url: string): Promise<string> => {
    try {
      let image = await axios.get(url, { responseType: "arraybuffer" });
      let raw = Buffer.from(image.data).toString("base64");
      return "data:" + image.headers["content-type"] + ";base64," + raw;
    } catch (error) {
      console.log(error);
      throw error;
    }
  };

  getOptions = (): BorderOptions => {
    let options = new BorderOptions();

    // output size
    let output_size = new OutputSize();
    output_size.width = this.state.outputWidth;
    output_size.height = this.state.outputHeight;
    options.output_size = output_size;

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

  update = (e?: React.FormEvent<HTMLFormElement>, force = false) => {
    e?.preventDefault();

    let config = {
      canvasScale: this.state.canvasScale,
      outputSizeName: this.state.outputSizeName,
      outputWidth: this.state.outputWidth,
      outputHeight: this.state.outputHeight,
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
      if (force || configHash !== this.lastRenderConfigHash) {
        this.scheduleUpdate(300);
      }
      return;
    }
    this.resize();

    this.setState({ rendering: true }, () => {
      this.lastRenderConfigHash = configHash;
      new Promise<void>((resolve, reject) => {
        const canvas = this.canvas.current;
        const previewSrcCanvas = this.previewSrcCanvas.current;
        if (!canvas || !previewSrcCanvas) return reject();
        const ctx = canvas.getContext("2d");
        const previewSrcCtx = previewSrcCanvas.getContext("2d");
        if (!ctx || !previewSrcCtx) return reject();

        let renderID = uuidv4();
        console.time(renderID);
        let imgData = ImageBorders.to_image_data(
          previewSrcCanvas,
          previewSrcCtx
        );
        this.worker.postMessage({ sourceImage: imgData });
        let options = this.getOptions();
        let size = new OutputSize();
        size.width = canvas.width;
        size.height = canvas.height;
        options.output_size = size;
        options.preview = true;
        this.worker.postMessage({
          applyOptions: options.serialize(),
          renderID,
          save: false,
        });
        return resolve();
      });
    });
  };

  exportHighRes = (e?: React.MouseEvent<HTMLButtonElement>) => {
    let originalSrcCanvas = this.originalSrcCanvas.current;
    let resultCanvas = this.resultCanvas.current;
    if (!resultCanvas || !originalSrcCanvas) return;
    const originalSrcCtx = originalSrcCanvas.getContext("2d");
    const resultCtx = resultCanvas.getContext("2d");
    if (!resultCtx || !originalSrcCtx) return;
    resultCanvas.width = this.state.outputWidth;
    resultCanvas.height = this.state.outputHeight;
    let renderID = uuidv4();
    console.time(renderID);
    let imgData = ImageBorders.to_image_data(originalSrcCanvas, originalSrcCtx);
    this.worker.postMessage({ sourceImage: imgData });
    let options = this.getOptions();
    options.preview = false;
    this.worker.postMessage({
      applyOptions: options.serialize(),
      renderID,
      save: true,
    });
  };

  save = (canvas: HTMLCanvasElement) => {
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

  loadOriginal = () => {
    let originalSrcCanvas = this.originalSrcCanvas.current;
    if (!originalSrcCanvas) return;
    originalSrcCanvas.width = this.img.width;
    originalSrcCanvas.height = this.img.height;
    originalSrcCanvas.getContext("2d")?.drawImage(this.img, 0, 0);
  };

  loadImage = (src: string) => {
    this.img = new Image();
    this.img.onload = () => {
      let originalSrcCanvas = this.originalSrcCanvas.current;
      let previewSrcCanvas = this.previewSrcCanvas.current;
      let canvas = this.canvas.current;
      if (!previewSrcCanvas || !originalSrcCanvas || !canvas) return;
      let previewScaledownFac =
        PREVIEW_MAX_RES / Math.max(this.img.width, this.img.height);
      canvas.width = this.img.width * previewScaledownFac;
      canvas.height = this.img.height * previewScaledownFac;
      originalSrcCanvas.width = this.img.width;
      originalSrcCanvas.height = this.img.height;
      previewSrcCanvas.width = canvas.width;
      previewSrcCanvas.height = canvas.height;

      previewSrcCanvas
        .getContext("2d")
        ?.drawImage(
          this.img,
          0,
          0,
          this.img.width,
          this.img.height,
          0,
          0,
          previewSrcCanvas.width,
          previewSrcCanvas.height
        );
      this.loadOriginal();
      this.resize();
      this.update(undefined, true);
    };
    this.img.src = src;
  };

  renderToCanvas = (img: ImageData, canvas: HTMLCanvasElement | null) => {
    canvas?.getContext("2d")?.putImageData(img, 0, 0);
  };

  resize = () => {
    let canvasContainer = this.canvasContainer.current;
    console.assert(canvasContainer);
    if (!canvasContainer) return;
    let canvasScale =
      Math.min(
        canvasContainer.clientWidth / this.state.outputWidth,
        canvasContainer.clientHeight / this.state.outputHeight
      ) * 0.95;
    this.setState({ canvasScale }, () => {
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
    });
  };

  scheduleUpdate = (timeout = 100) => {
    clearTimeout(this.updateTimer);
    this.updateTimer = setTimeout(this.update, timeout);
  };

  scheduleResize = () => {
    clearTimeout(this.resizeTimer);
    this.resizeTimer = setTimeout(() => {
      console.log("resize");
      this.resize();
      this.update(undefined, false);
    }, 300);
  };

  componentDidMount = async () => {
    this.worker = new Worker(
      `${process.env.PUBLIC_URL}/worker/ImageBorder.worker.js`
    );
    this.worker.onmessage = (event) => {
      // console.log("message from worker: ", event);
      if ("status" in event.data) {
        if (event.data.status === "ready") this.setState({ workerReady: true });
      }
      if ("result" in event.data) {
        if (event.data.save) {
          let resultCanvas = this.resultCanvas.current;
          if (resultCanvas) {
            this.renderToCanvas(event.data.result, resultCanvas);
            this.save(resultCanvas);
            this.update(undefined);
          }
        } else {
          this.renderToCanvas(event.data.result, this.canvas.current);
        }
        console.timeEnd(event.data.renderID);
        this.setState({ rendering: false });
      }
    };

    await this.loadWasm();
    let sampleImage = await this.getB64Image(
      "https://upload.wikimedia.org/wikipedia/commons/thumb/4/4c/Brad_Pitt_2019_by_Glenn_Francis.jpg/1200px-Brad_Pitt_2019_by_Glenn_Francis.jpg"
    );
    await this.loadImage(sampleImage);

    window.addEventListener("resize", this.scheduleResize, false);
  };

  componentWillUnmount() {
    window.removeEventListener("resize", this.scheduleResize, false);
  }

  stripExtension = (filename: string): string => {
    return filename.replace(/\.[^/.]+$/, "");
  };

  openImage = (files: FileList | null) => {
    if (!files || files.length < 1) return;
    this.setState({
      filename: `${this.stripExtension(files[0].name)}_border.png`,
    });
    console.log(`loading ${files[0]}...`);
    let reader = new FileReader();
    reader.onload = () => {
      this.loadImage(reader.result as string);
    };
    reader.readAsDataURL(files[0]);
  };

  updateRotationAngle = (e: React.ChangeEvent<HTMLSelectElement>) => {
    this.setState(
      {
        // @ts-ignore
        rotationAngle: Rotation[e.target.value],
        // @ts-ignore
        rotationAngleName: Rotation[Rotation[e.target.value]],
      },
      () => this.scheduleUpdate()
    );
  };

  updateOutputSize = (e: React.ChangeEvent<HTMLSelectElement>) => {
    let key = e.target.value as keyof typeof OUTPUT_SIZES;
    if (key in OUTPUT_SIZES) {
      this.setState(
        {
          outputSizeName: e.target.value,
          outputWidth: OUTPUT_SIZES[key].width,
          outputHeight: OUTPUT_SIZES[key].height,
        },
        () => this.scheduleUpdate()
      );
    } else {
      this.setState(
        {
          outputSizeName: e.target.value,
        },
        () => this.scheduleUpdate()
      );
    }
  };

  updateOutputWidth = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ outputWidth: parseFloat(e.target.value) }, () =>
      this.scheduleUpdate()
    );
  };

  updateOutputHeight = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ outputHeight: parseFloat(e.target.value) }, () =>
      this.scheduleUpdate()
    );
  };

  updateScaleFactor = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ scaleFactor: parseFloat(e.target.value) }, () =>
      this.scheduleUpdate()
    );
  };

  updateBorderWidth = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState(
      {
        borderWidthTop: parseFloat(e.target.value),
        borderWidthRight: parseFloat(e.target.value),
        borderWidthBottom: parseFloat(e.target.value),
        borderWidthLeft: parseFloat(e.target.value),
      },
      () => this.scheduleUpdate()
    );
  };

  updateCropTop = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ cropTop: parseFloat(e.target.value) }, () =>
      this.scheduleUpdate()
    );
  };
  updateCropRight = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ cropRight: parseFloat(e.target.value) }, () =>
      this.scheduleUpdate()
    );
  };
  updateCropBottom = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ cropBottom: parseFloat(e.target.value) }, () =>
      this.scheduleUpdate()
    );
  };
  updateCropLeft = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ cropLeft: parseFloat(e.target.value) }, () =>
      this.scheduleUpdate()
    );
  };

  render = () => {
    return (
      <div id="app">
        <canvas className="offscreen" ref={this.resultCanvas}></canvas>
        <canvas className="offscreen" ref={this.previewSrcCanvas}></canvas>
        <canvas className="offscreen" ref={this.originalSrcCanvas}></canvas>

        <div id="wasm-canvas-container" ref={this.canvasContainer}>
          <Oval
            wrapperClass={
              "spinner " +
              (this.state.rendering ||
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
              {/*disabled={this.state.rendering}*/}
              <select
                id="outputSize"
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
                disabled={this.state.outputSizeName !== "Custom"}
                value={this.state.outputWidth}
                onChange={this.updateOutputWidth}
              />
              <label htmlFor="outputWidth">Width</label>

              <input
                id="outputHeight"
                type="number"
                step="1"
                disabled={this.state.outputSizeName !== "Custom"}
                value={this.state.outputHeight}
                onChange={this.updateOutputHeight}
              />
              <label htmlFor="outputHeight">Height</label>

              <select
                id="rotationAngle"
                value={this.state.rotationAngleName}
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
                value={this.state.scaleFactor}
                onChange={this.updateScaleFactor}
              />
              <label htmlFor="scaleFactor">Scale factor</label>

              <input
                type="number"
                id="borderWidth"
                value={this.state.borderWidthTop}
                onChange={this.updateBorderWidth}
              />
              <label htmlFor="borderWidth">Border width</label>

              <input
                type="number"
                id="cropTop"
                value={this.state.cropTop}
                onChange={this.updateCropTop}
              />
              <label htmlFor="cropTop">Crop top</label>

              <input
                type="number"
                id="cropRight"
                value={this.state.cropRight}
                onChange={this.updateCropRight}
              />
              <label htmlFor="cropRight">Crop right</label>

              <input
                type="number"
                id="cropBottom"
                value={this.state.cropBottom}
                onChange={this.updateCropBottom}
              />
              <label htmlFor="cropBottom">Crop bottom</label>

              <input
                type="number"
                id="cropLeft"
                value={this.state.cropLeft}
                onChange={this.updateCropLeft}
              />
              <label htmlFor="cropLeft">Crop left</label>

              <button disabled={this.state.rendering} type="submit">
                Update
              </button>
              <button
                disabled={this.state.rendering}
                onClick={this.exportHighRes}
              >
                Export
              </button>
              <input
                type="file"
                disabled={this.state.rendering}
                onChange={(e) => this.openImage(e.target.files)}
                name="fileinput"
                id="fileinput"
                accept="image/*"
              />
            </div>
          </fieldset>
        </form>
      </div>
    );
  };
}
