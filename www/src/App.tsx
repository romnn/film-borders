import React from "react";
import logo from "./logo.svg";
import "./App.css";

type AppState = {
  wasm?: typeof import("wasm-mod");
};

type AppProps = {};

export default class App extends React.Component<AppProps, AppState> {
  constructor(props: AppProps) {
    super(props);
    this.state = {
      wasm: undefined,
    };
  }

  loadWasm = async () => {
    try {
      const wasm = await import("wasm-mod");
      this.setState({ wasm });
    } catch (err) {
      console.error(`unexpected error when loading WASM. (${err.message})`);
    }
  };


  loadImage = async() => {
    const canvas = this.refs.canvas;
    const ctx = canvas.getContext("2d");
    const processsor = this.state.wasm;
    ctx.drawImage(this.img, 0, 0);
    let img = processor.open_image(canvas, ctx);

    // console.time("PHOTON_WITH_RAWPIX");
    // photon.alter_channel(phtimg, 2, 70);
    // photon.grayscale(phtimg);
    // console.timeEnd("PHOTON_WITH_RAWPIX");

    // // Replace the current canvas' ImageData with the new image's ImageData.
    // update the canvas image
    processor.putImageData(canvas, ctx, img);



    // console.time("PHOTON_CONSTR");
    // photon.canvas_wasm_only(canvas1, ctx);
    // console.timeEnd("PHOTON_CONSTR");
  }

  componentDidMount = () => {
    this.loadWasm().then(() => {
      this.state.wasm?.greet("roman here");
    });
  };

  render = () => {
    return (
      <div className="App">
        <header className="App-header">
        </header>
        <main>
          <canvas ref="canvas"></canvas>
        </main>
      </div>
    );
  };
}
