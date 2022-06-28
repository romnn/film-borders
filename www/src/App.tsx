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

type AppState = {
  wasmLoaded: boolean;
  workerReady: boolean;
  rendering: boolean;
  filename?: string;
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
const OUTPUT_SIZES = {
  Portrait: {
    width: 1080,
    height: 1350,
  },
  Landscape: {
    width: 1080,
    height: 608,
  },
  Square: {
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
  // protected wasm!: typeof import("filmborders");
  protected wasm!: typeof import("filmborders");
  protected worker!: Worker;
  protected img!: HTMLImageElement;

  constructor(props: AppProps) {
    super(props);
    this.state = {
      wasmLoaded: false,
      workerReady: false,
      rendering: true,
      filename: undefined,
      outputSizeName: "Portrait",
      outputWidth: OUTPUT_SIZES["Portrait"].width,
      outputHeight: OUTPUT_SIZES["Portrait"].height,
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

  update = (e?: React.FormEvent<HTMLFormElement>) => {
    e?.preventDefault();
    this.setState({ rendering: true }, () => {
      new Promise<void>((resolve, reject) => {
        const canvas = this.canvas.current;
        const previewSrcCanvas = this.previewSrcCanvas.current;
        if (!canvas || !previewSrcCanvas) return reject();
        const ctx = canvas.getContext("2d");
        const previewSrcCtx = previewSrcCanvas.getContext("2d");
        if (!ctx || !previewSrcCtx) return reject();

        let canvasContainer = this.canvasContainer.current;
        let canvasScale = Math.min(
          (canvasContainer?.clientWidth ?? 0) / this.state.outputWidth,
          (canvasContainer?.clientHeight ?? 0) / this.state.outputHeight
        );
        canvas.width = this.state.outputWidth * canvasScale;
        canvas.height = this.state.outputHeight * canvasScale;

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
      this.update(undefined);
    };
    this.img.src = src;
  };

  renderToCanvas = (img: ImageData, canvas: HTMLCanvasElement | null) => {
    canvas?.getContext("2d")?.putImageData(img, 0, 0);
  };

  handleResize = () => {
    let canvasContainer = this.canvasContainer.current;
    let canvas = this.canvas.current;
    if (!canvasContainer || !canvas) return;
    let canvasScale = Math.min(
      canvasContainer.clientWidth / this.state.outputWidth,
      canvasContainer.clientHeight / this.state.outputHeight
    );
    // changing the size of the canvas causes it to go white but thats fine
    canvas.width = this.state.outputWidth * canvasScale;
    canvas.height = this.state.outputHeight * canvasScale;
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
    // .then((sampleImage) => this.loadImage(sampleImage));
    // });

    window.addEventListener("resize", this.handleResize);
  };

  componentWillUnmount() {
    window.removeEventListener("resize", this.handleResize);
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
    // @ts-ignore
    this.setState({ rotationAngle: Rotation[e.target.value] });
    this.setState({
      // @ts-ignore
      rotationAngleName: Rotation[Rotation[e.target.value]],
    });
  };

  updateOutputSize = (e: React.ChangeEvent<HTMLSelectElement>) => {
    this.setState({
      outputSizeName: e.target.value,
      outputWidth:
        OUTPUT_SIZES[e.target.value as keyof typeof OUTPUT_SIZES].width,
      outputHeight:
        OUTPUT_SIZES[e.target.value as keyof typeof OUTPUT_SIZES].height,
    });
  };

  updateScaleFactor = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ scaleFactor: parseFloat(e.target.value) });
  };

  updateBorderWidth = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({
      borderWidthTop: parseFloat(e.target.value),
      borderWidthRight: parseFloat(e.target.value),
      borderWidthBottom: parseFloat(e.target.value),
      borderWidthLeft: parseFloat(e.target.value),
    });
  };

  updateCropTop = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ cropTop: parseFloat(e.target.value) });
  };
  updateCropRight = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ cropRight: parseFloat(e.target.value) });
  };
  updateCropBottom = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ cropBottom: parseFloat(e.target.value) });
  };
  updateCropLeft = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.setState({ cropLeft: parseFloat(e.target.value) });
  };

  render = () => {
    return (
      <div id="app">
        <header></header>
        <main>
          <canvas className="offscreen" ref={this.resultCanvas}></canvas>
          <canvas className="offscreen" ref={this.previewSrcCanvas}></canvas>
          <canvas className="offscreen" ref={this.originalSrcCanvas}></canvas>
          <div id="wasm-canvas-container" ref={this.canvasContainer}>
            <Oval
              visible={
                this.state.rendering ||
                !this.state.wasmLoaded ||
                !this.state.workerReady
              }
              color="#00BFFF"
              height={100}
              width={100}
            />
            <canvas id="wasm-canvas" ref={this.canvas}></canvas>
          </div>
          <form className="parameters" onSubmit={this.update}>
            <fieldset>
              <legend>Film Borders!</legend>

              <div className="formgrid">
                <select
                  id="outputSize"
                  disabled={this.state.rendering}
                  value={this.state.outputSizeName}
                  onChange={this.updateOutputSize}
                >
                  {Object.keys(OUTPUT_SIZES).map((option) => (
                    <option value={option} key={option}>
                      {option}
                    </option>
                  ))}
                </select>
                <label htmlFor="outputSize">Size</label>

                <select
                  id="rotationAngle"
                  disabled={this.state.rendering}
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
                  disabled={this.state.rendering}
                  value={this.state.scaleFactor}
                  onChange={this.updateScaleFactor}
                />
                <label htmlFor="scaleFactor">Scale factor</label>

                <input
                  type="number"
                  id="borderWidth"
                  disabled={this.state.rendering}
                  value={this.state.borderWidthTop}
                  onChange={this.updateBorderWidth}
                />
                <label htmlFor="borderWidth">Border width</label>

                <input
                  type="number"
                  id="cropTop"
                  disabled={this.state.rendering}
                  value={this.state.cropTop}
                  onChange={this.updateCropTop}
                />
                <label htmlFor="cropTop">Crop top</label>

                <input
                  type="number"
                  id="cropRight"
                  disabled={this.state.rendering}
                  value={this.state.cropRight}
                  onChange={this.updateCropRight}
                />
                <label htmlFor="cropRight">Crop right</label>

                <input
                  type="number"
                  id="cropBottom"
                  disabled={this.state.rendering}
                  value={this.state.cropBottom}
                  onChange={this.updateCropBottom}
                />
                <label htmlFor="cropBottom">Crop bottom</label>

                <input
                  type="number"
                  id="cropLeft"
                  disabled={this.state.rendering}
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
        </main>
      </div>
    );
  };
}
