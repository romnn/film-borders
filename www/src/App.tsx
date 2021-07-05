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

  componentDidMount = () => {
    this.loadWasm().then(() => {
      this.state.wasm?.greet("roman here");
    });
  };

  render = () => {
    return (
      <div className="App">
        <header className="App-header">
          <img src={logo} className="App-logo" alt="logo" />
          <p>
            Edit <code>src/App.tsx</code> and save to reload.
          </p>
          <a
            className="App-link"
            href="https://reactjs.org"
            target="_blank"
            rel="noopener noreferrer"
          >
            Learn React
          </a>
        </header>
      </div>
    );
  };
}
