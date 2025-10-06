import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

import '@mantine/core/styles.css';
import { MantineProvider } from "@mantine/core";
import { GoogleOAuthProvider } from "@react-oauth/google";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <GoogleOAuthProvider clientId="579843454461-cfehvkto8uoki7ahkopg2e55fr3sl9fe.apps.googleusercontent.com">
      <MantineProvider defaultColorScheme="auto">
        <App />
      </MantineProvider>
    </GoogleOAuthProvider>
  </React.StrictMode>,
);
