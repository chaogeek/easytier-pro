import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { SWRConfig } from "swr";
import { ThemeProvider, createTheme, CssBaseline } from "@mui/material";
import App from "./App";

// MUI 主题
const theme = createTheme({
  palette: {
    mode: "light",
    primary: {
      main: "#1976d2",
    },
  },
  typography: {
    fontFamily: [
      "-apple-system",
      "BlinkMacSystemFont",
      '"Segoe UI"',
      "Roboto",
      '"Helvetica Neue"',
      "Arial",
      "sans-serif",
    ].join(","),
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: "none",
        },
      },
    },
  },
});

// SWR 全局配置
const swrConfig = {
  // 默认每 30 秒轮询一次（用于实时数据如延迟、流量）
  refreshInterval: 30000,
  // 窗口重新聚焦时刷新
  revalidateOnFocus: true,
  // 出错时 5 秒后重试
  errorRetryInterval: 5000,
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <SWRConfig value={swrConfig}>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </SWRConfig>
    </ThemeProvider>
  </React.StrictMode>
);
