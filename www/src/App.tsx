import React from "react";
import logo from "./logo.svg";
import "./App.css";
import axios from "axios";

type AppState = {};
type AppProps = {};

export default class App extends React.Component<AppProps, AppState> {
  protected offscreenCanvas = React.createRef<HTMLCanvasElement>();
  protected canvas = React.createRef<HTMLCanvasElement>();
  protected wasm!: typeof import("wasm-mod");
  protected img!: HTMLImageElement;

  constructor(props: AppProps) {
    super(props);
    this.state = {};
  }

  loadWasm = async () => {
    try {
      this.wasm = await import("wasm-mod");
    } catch (err) {
      console.error(`unexpected error when loading WASM. (${err.message})`);
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

  update = () => {
    const canvas = this.canvas.current;
    const offscreenCanvas = this.offscreenCanvas.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    const offscreenCtx = offscreenCanvas.getContext("2d");
    if (!ctx) return;
    console.time("wasm render");
    let wasmImg = new this.wasm.WasmImageBorders(offscreenCanvas, offscreenCtx);
    let options = new this.wasm.ImageBorderOptions();
    console.log(options);
    wasmImg.apply(options);
    wasmImg.update(canvas, ctx);
    console.timeEnd("wasm render");
  };

  loadImage = async () => {
    this.img = new Image();
    this.img.onload = () => {
      console.log("drawing image");
      let offscreenCanvas = this.offscreenCanvas.current;
      let canvas = this.canvas.current;
      canvas.width = this.img.width;
      canvas.height = this.img.height;
      offscreenCanvas.width = this.img.width;
      offscreenCanvas.height = this.img.height;
      offscreenCanvas?.getContext("2d").drawImage(this.img, 0, 0);
      // canvas
      //   ?.getContext("2d")
      //   ?.drawImage(
      //     this.img,
      //     0,
      //     0,
      //     this.img.width,
      //     this.img.height,
      //     0,
      //     0,
      //     canvas.width,
      //     canvas.height
      //   );
      this.update();
    };
    this.img.src = await this.getB64Image(
      "https://upload.wikimedia.org/wikipedia/commons/thumb/4/4c/Brad_Pitt_2019_by_Glenn_Francis.jpg/1200px-Brad_Pitt_2019_by_Glenn_Francis.jpg"
    );
  };

  componentDidMount = () => {
    this.loadWasm().then(() => {
      this.loadImage();
    });
  };

  render = () => {
    return (
      <div id="app">
        <header></header>
        <main>
          <canvas
            id="wasm-offscreen-canvas"
            ref={this.offscreenCanvas}
          ></canvas>
          <canvas id="wasm-canvas" ref={this.canvas}></canvas>
          <div className="parameters"></div>
        </main>
      </div>
    );
  };
}
