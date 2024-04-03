import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "react-redux";
import { ChakraProvider } from "@chakra-ui/react";
import store from "./store/store";
import App from "./App";
import "./styles.css";
import { Modals } from "./components/Modals";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider store={store}>
      <ChakraProvider>
        <Modals />
        <App />
      </ChakraProvider>
    </Provider>
  </React.StrictMode>
);
