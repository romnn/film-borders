import React from "react";
import { Oval } from "react-loader-spinner";
import init, {
  Sides,
  FitMode,
  Options,
  BoundedSize,
  Rotation,
  Builtin,
  Color,
} from "filmborders";
import "./App.sass";
import hash from "object-hash";

type AppState = {
  wasmLoaded: boolean;
  workerReady: boolean;
  rendering: boolean;
  exporting: boolean;
  filename?: string;
  fitMode?: FitMode;
  fitModeName?: string;
  borderOverlay?: Builtin;
  borderOverlayName?: string;
  canvasScale: number;
  outputSizeName: string;
  outputWidth: number;
  outputHeight: number;
  frameColor: string;
  backgroundColor: string;
  scaleFactor: number;
  preview: boolean;
  margin: number;
  cropTop?: number;
  cropRight?: number;
  cropBottom?: number;
  cropLeft?: number;
  frameWidthTop?: number;
  frameWidthRight?: number;
  frameWidthBottom?: number;
  frameWidthLeft?: number;
  imageRotation?: Rotation;
  imageRotationName?: string;
  borderRotation?: Rotation;
  borderRotationName?: string;
};

const MAX_PREVIEW_DIM = 1000;
const DEFAULT_BORDER_WIDTH_PERCENT = 10;

const OUTPUT_SIZES_KEYS: string[] = [
  "Match source",
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
  protected updateTimer?: ReturnType<typeof setTimeout>;
  protected lastRenderConfigHash?: string;
  protected maxPreviewDim = MAX_PREVIEW_DIM;

  constructor(props: AppProps) {
    super(props);
    const outputSizeName = "Match source";
    this.state = {
      wasmLoaded: false,
      workerReady: false,
      rendering: false,
      exporting: false,
      filename: undefined,
      fitMode: undefined,
      fitModeName: undefined,
      borderOverlay: undefined,
      borderOverlayName: undefined,
      canvasScale: 0.0,
      outputSizeName,
      outputWidth: 0,
      outputHeight: 0,
      frameColor: "#000000",
      backgroundColor: "#ffffff",
      scaleFactor: 100.0,
      preview: false,
      margin: 10.0,
      cropTop: 0.0,
      cropRight: 0.0,
      cropBottom: 0.0,
      cropLeft: 0.0,
      frameWidthTop: DEFAULT_BORDER_WIDTH_PERCENT,
      frameWidthRight: DEFAULT_BORDER_WIDTH_PERCENT,
      frameWidthBottom: DEFAULT_BORDER_WIDTH_PERCENT,
      frameWidthLeft: DEFAULT_BORDER_WIDTH_PERCENT,
      imageRotation: undefined,
      imageRotationName: undefined,
      borderRotation: undefined,
      borderRotationName: undefined,
    };
  }

  setWasmDefaults = async () => {
    const borderOverlay = Builtin.Border120_1;
    const defaultRotation = Rotation.Rotate0;
    const fitMode = FitMode.Image;

    await this.setState({
      imageRotation: defaultRotation,
      imageRotationName: Rotation[defaultRotation],
      borderRotation: defaultRotation,
      borderRotationName: Rotation[defaultRotation],
      fitMode,
      fitModeName: FitMode[fitMode],
      borderOverlay,
      borderOverlayName: Builtin[borderOverlay],
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

  getOptions = async (): Promise<Options> => {
    let options = new Options();

    // output size
    let output_size = new BoundedSize();
    if (this.state.outputSizeName !== "Match source") {
      output_size.width = this.state.outputWidth;
      output_size.height = this.state.outputHeight;
    }
    options.output_size = output_size;

    // fit mode
    options.mode = this.state.fitMode ?? FitMode.Image;

    // frame color
    try {
      options.frame_color = new Color(this.state.frameColor);
    } catch (error) {
      console.error(error);
    }

    // background color
    try {
      options.background_color = new Color(this.state.backgroundColor);
    } catch (error) {
      console.error(error);
    }

    // scale factor
    options.scale_factor = (this.state.scaleFactor ?? 100.0) / 100.0;

    // preview
    options.preview = this.state.preview;

    // margin
    options.margin = (this.state.margin ?? 0.0) / 100.0;

    // crop
    let crop = new Sides();
    crop.top = (this.state.cropTop ?? 0.0) / 100.0;
    crop.right = (this.state.cropRight ?? 0.0) / 100.0;
    crop.bottom = (this.state.cropBottom ?? 0.0) / 100.0;
    crop.left = (this.state.cropLeft ?? 0.0) / 100.0;
    options.crop = crop;

    // border width
    let frameWidth = new Sides();
    frameWidth.top = (this.state.frameWidthTop ?? 0.0) / 100.0;
    frameWidth.right = (this.state.frameWidthRight ?? 0.0) / 100.0;
    frameWidth.bottom = (this.state.frameWidthBottom ?? 0.0) / 100.0;
    frameWidth.left = (this.state.frameWidthLeft ?? 0.0) / 100.0;
    options.frame_width = frameWidth;

    // rotation
    options.image_rotation = this.state.imageRotation ?? Rotation.Rotate0;
    options.border_rotation = this.state.borderRotation ?? Rotation.Rotate0;
    return options;
  };

  update = async (e?: React.FormEvent<HTMLFormElement>, force = false) => {
    if (!this.state.workerReady) return;
    try {
      console.log("render");

      e?.preventDefault();

      let config = {
        borderOverlay: this.state.borderOverlay,
        borderOverlayName: this.state.borderOverlayName,
        fitMode: this.state.fitMode,
        fitModeName: this.state.fitModeName,
        canvasScale: this.state.canvasScale,
        outputSizeName: this.state.outputSizeName,
        outputWidth: this.state.outputWidth,
        outputHeight: this.state.outputHeight,
        frameColor: this.state.frameColor,
        backgroundColor: this.state.backgroundColor,
        scaleFactor: this.state.scaleFactor,
        preview: this.state.preview,
        margin: this.state.margin,
        cropTop: this.state.cropTop,
        cropRight: this.state.cropRight,
        cropBottom: this.state.cropBottom,
        cropLeft: this.state.cropLeft,
        frameWidthTop: this.state.frameWidthTop,
        frameWidthRight: this.state.frameWidthTop,
        frameWidthBottom: this.state.frameWidthBottom,
        frameWidthLeft: this.state.frameWidthLeft,
        imageRotationName: this.state.imageRotationName,
        imageRotation: this.state.imageRotation,
        borderRotationName: this.state.borderRotationName,
        borderRotation: this.state.borderRotation,
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

      let canvasContainer = this.canvasContainer.current;

      if (canvasContainer) {
        let bounds = new BoundedSize();
        let scale = 0.95;
        bounds.width = canvasContainer.clientWidth * scale;
        bounds.height = canvasContainer.clientHeight * scale;
        options.output_size_bounds = bounds;
      }

      await this.worker.postMessage({
        borderName: this.state.borderOverlay,
        options: options.serialize(),
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

    let borderCanvas = this.borderCanvas.current;
    let originalCanvas = this.originalCanvas.current;
    let resultCanvas = this.resultCanvas.current;
    if (!resultCanvas || !originalCanvas || !borderCanvas) return;

    const borderCtx = borderCanvas.getContext("2d");
    const originalCtx = originalCanvas.getContext("2d");
    const resultCtx = resultCanvas.getContext("2d");
    if (!resultCtx || !originalCtx || !borderCtx) return;

    // resultCanvas.width = this.state.outputWidth;
    // resultCanvas.height = this.state.outputHeight;
    let renderID = uuidv4();
    console.time(renderID);

    let imageData = originalCtx.getImageData(
      0,
      0,
      originalCanvas.width,
      originalCanvas.height
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
    // let size = new BoundedSize();
    // size.width = resultCanvas.width;
    // size.height = resultCanvas.height;
    // options.output_size = size;
    // options.output_size.width = this.state.outputWidth;
    // options.output_size.height = this.state.outputHeight;
    options.preview = false;

    // console.log(options);
    await this.worker.postMessage({
      borderName: this.state.borderOverlay,
      options: options.serialize(),
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
        // canvas.width = width;
        // canvas.height = height;
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
        // let canvas = this.canvas.current;
        // if (!previewCanvas || !originalCanvas || !canvas) return reject();
        if (!previewCanvas || !originalCanvas) return reject();

        this.maxPreviewDim = this.computeMaxPreviewDim(width, height);
        let maxImageDim = Math.max(width, height);
        let previewScale = this.maxPreviewDim / maxImageDim;

        // canvas.width = width * previewScaledownFac;
        // canvas.height = height * previewScaledownFac;

        originalCanvas.width = width;
        originalCanvas.height = height;
        previewCanvas.width = width * previewScale;
        previewCanvas.height = height * previewScale;

        // previewCanvas.width = canvas.width;
        // previewCanvas.height = canvas.height;

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
    if (!canvas) return;
    canvas.width = img.width;
    canvas.height = img.height;
    canvas?.getContext("2d")?.putImageData(img, 0, 0);
  };

  maxScreenDim = (): number => {
    const screenWidth =
      window.innerWidth ||
      document.documentElement.clientWidth ||
      document.body.clientWidth;
    const screenHeight =
      window.innerHeight ||
      document.documentElement.clientHeight ||
      document.body.clientHeight;

    return Math.max(screenWidth, screenHeight);
  };

  computeMaxPreviewDim = (imgWidth: number, imgHeight: number): number => {
    let maxScreenDim = this.maxScreenDim();
    let maxImageDim = Math.max(imgWidth, imgHeight);
    let maxPreviewDim = Math.min(MAX_PREVIEW_DIM, maxScreenDim);
    maxPreviewDim = Math.min(maxPreviewDim, maxImageDim);
    return maxPreviewDim;
  };

  scheduleUpdate = async (timeout = 100) => {
    clearTimeout(this.updateTimer);
    this.updateTimer = setTimeout(this.update, timeout);
  };

  handleResize = async () => {
    await this.scheduleUpdate();
    // clearTimeout(this.resizeTimer);
    // this.resizeTimer = setTimeout(async () => {
    //   console.log("resize");
    //   await this.update(undefined, false);
    // }, 300);
  };

  // scheduleResize = async () => {
  //   clearTimeout(this.resizeTimer);
  //   this.resizeTimer = setTimeout(async () => {
  //     console.log("resize");
  //     await this.update(undefined, false);
  //   }, 300);
  // };

  componentDidMount = async () => {
    this.worker = new Worker(
      `${process.env.PUBLIC_URL}/worker/ImageBorder.worker.js`
    );
    this.worker.onmessage = async (event) => {
      if ("status" in event.data) {
        if (event.data.status === "ready") {
          console.log("worker ready");
          await this.setState({ workerReady: true });
          await this.scheduleUpdate();
        }
      }
      if ("result" in event.data) {
        let result = event.data.result;
        if (event.data.save) {
          let resultCanvas = this.resultCanvas.current;
          if (resultCanvas) {
            await this.renderToCanvas(result, resultCanvas);
            await this.save(resultCanvas);
            await this.update(undefined);
          }
          await this.setState({ exporting: false });
        } else {
          await this.renderToCanvas(result, this.canvas.current);
        }
        console.timeEnd(event.data.renderID);
        await this.setState({ rendering: false });
      }
    };

    await this.init();
    window.addEventListener("resize", this.handleResize, false);
  };

  componentWillUnmount() {
    window.removeEventListener("resize", this.handleResize, false);
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

  updateFitMode = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    let key = e.target.value as keyof typeof FitMode;
    if (key in FitMode) {
      await this.setState({
        fitMode: FitMode[key],
        fitModeName: FitMode[FitMode[key]],
      });
    } else {
      await this.setState({
        fitMode: undefined,
      });
    }
    await this.scheduleUpdate();
  };

  updateBorderOverlay = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    let key = e.target.value as keyof typeof Builtin;
    if (key in Builtin) {
      await this.setState({
        borderOverlay: Builtin[key],
        borderOverlayName: Builtin[Builtin[key]],
      });
    } else {
      await this.setState({
        borderOverlay: undefined,
        borderOverlayName: key,
      });
    }
    await this.scheduleUpdate();
  };

  updateBorderRotation = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    let key = e.target.value as keyof typeof Rotation;
    await this.setState({
      borderRotation: Rotation[key],
      borderRotationName: Rotation[Rotation[key]],
    });
    await this.scheduleUpdate();
  };

  updateImageRotation = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    let key = e.target.value as keyof typeof Rotation;
    await this.setState({
      imageRotation: Rotation[key],
      imageRotationName: Rotation[Rotation[key]],
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

  updateFrameColor = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ frameColor: e.target.value });
    await this.scheduleUpdate();
  };

  updateBackgroundColor = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ backgroundColor: e.target.value });
    await this.scheduleUpdate();
  };

  updateMargin = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ margin: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };

  updateScaleFactor = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ scaleFactor: parseFloat(e.target.value) });
    await this.scheduleUpdate();
  };

  updatePreview = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({ preview: e.target.checked });
    await this.scheduleUpdate();
  };

  updateframeWidth = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await this.setState({
      frameWidthTop: parseFloat(e.target.value),
      frameWidthRight: parseFloat(e.target.value),
      frameWidthBottom: parseFloat(e.target.value),
      frameWidthLeft: parseFloat(e.target.value),
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
              <p id="app-title">film borders.</p>
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
                id="fitMode"
                value={this.state.fitModeName}
                disabled={this.state.exporting}
                onChange={this.updateFitMode}
              >
                {Object.values(FitMode)
                  .filter((r) => typeof r == "string")
                  .map((option) => (
                    <option value={option} key={option}>
                      {option}
                    </option>
                  ))}
              </select>
              <label htmlFor="fitMode">Fit Mode</label>

              <select
                id="borderOverlay"
                value={this.state.borderOverlayName}
                disabled={this.state.exporting}
                onChange={this.updateBorderOverlay}
              >
                {Object.values(Builtin)
                  .filter((r) => typeof r == "string")
                  .map((option) => (
                    <option value={option} key={option}>
                      {option}
                    </option>
                  ))}
                <option value="Custom" key="Custom">
                  Custom
                </option>
                <option value="None" key="None">
                  None
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
                className={
                  this.state.outputSizeName === "Match source" ? "hidden" : ""
                }
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
                className={
                  this.state.outputSizeName === "Match source" ? "hidden" : ""
                }
                disabled={
                  this.state.exporting || this.state.outputSizeName !== "Custom"
                }
                value={this.state.outputHeight}
                onChange={this.updateOutputHeight}
              />
              <label htmlFor="outputHeight">Height</label>

              <input
                id="frameColor"
                type="color"
                disabled={this.state.exporting}
                value={this.state.frameColor}
                onChange={this.updateFrameColor}
              />
              <label htmlFor="frameColor">Frame Color</label>

              <input
                id="backgroundColor"
                type="color"
                disabled={this.state.exporting}
                value={this.state.backgroundColor}
                onChange={this.updateBackgroundColor}
              />
              <label htmlFor="backgroundColor">Canvas Color</label>

              <select
                id="borderRotation"
                value={this.state.borderRotationName}
                disabled={this.state.exporting}
                onChange={this.updateBorderRotation}
              >
                {Object.values(Rotation)
                  .filter((r) => typeof r == "string")
                  .map((option) => (
                    <option value={option} key={option}>
                      {option}
                    </option>
                  ))}
              </select>
              <label htmlFor="borderRotation">Border Rotation</label>

              <select
                id="imageRotation"
                value={this.state.imageRotationName}
                disabled={this.state.exporting}
                onChange={this.updateImageRotation}
              >
                {Object.values(Rotation)
                  .filter((r) => typeof r == "string")
                  .map((option) => (
                    <option value={option} key={option}>
                      {option}
                    </option>
                  ))}
              </select>
              <label htmlFor="imageRotation">Image Rotation</label>

              <input
                id="scaleFactor"
                type="number"
                step="0.5"
                disabled={this.state.exporting}
                value={this.state.scaleFactor}
                onChange={this.updateScaleFactor}
              />
              <label htmlFor="scaleFactor">Scale %</label>

              <input
                id="margin"
                type="number"
                step="0.5"
                disabled={this.state.exporting}
                value={this.state.margin}
                onChange={this.updateMargin}
              />
              <label htmlFor="margin">Margin %</label>

              <input
                type="number"
                id="frameWidth"
                step="0.5"
                disabled={this.state.exporting}
                value={this.state.frameWidthTop}
                onChange={this.updateframeWidth}
              />
              <label htmlFor="frameWidth">Frame width %</label>

              <input
                id="cropTop"
                type="number"
                step="0.5"
                disabled={this.state.exporting}
                value={this.state.cropTop}
                onChange={this.updateCropTop}
              />
              <label htmlFor="cropTop">Crop top %</label>

              <input
                id="cropRight"
                type="number"
                step="0.5"
                disabled={this.state.exporting}
                value={this.state.cropRight}
                onChange={this.updateCropRight}
              />
              <label htmlFor="cropRight">Crop right %</label>

              <input
                id="cropBottom"
                type="number"
                step="0.5"
                disabled={this.state.exporting}
                value={this.state.cropBottom}
                onChange={this.updateCropBottom}
              />
              <label htmlFor="cropBottom">Crop bottom %</label>

              <input
                id="cropLeft"
                type="number"
                step="0.5"
                disabled={this.state.exporting}
                value={this.state.cropLeft}
                onChange={this.updateCropLeft}
              />
              <label htmlFor="cropLeft">Crop left %</label>

              <input
                id="preview"
                type="checkbox"
                disabled={this.state.exporting}
                checked={this.state.preview}
                onChange={this.updatePreview}
              />
              <label htmlFor="preview">insta visible</label>

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
                by <a href="https://romnn.com">romnn</a>
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
