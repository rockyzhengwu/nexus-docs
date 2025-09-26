import "./index.css";
import "./App.jsx";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "@/App";
import Home from "@/pages/home.jsx";
import { BrowserRouter, Route, Routes } from "react-router";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<App />}>
          <Route index element={<Home />}></Route>
        </Route>
      </Routes>
    </BrowserRouter>
  </StrictMode>,
);
