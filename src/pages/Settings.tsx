import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Typography,
  Box,
  Card,
  CardContent,
  Button,
  CircularProgress,
  Chip,
  LinearProgress,
} from "@mui/material";
import UpdateIcon from "@mui/icons-material/SystemUpdateAlt";
import RestartIcon from "@mui/icons-material/RestartAlt";
import DownloadIcon from "@mui/icons-material/Download";
import AppIcon from "@mui/icons-material/Apps";
import CoreIcon from "@mui/icons-material/Memory";
import ErrorIcon from "@mui/icons-material/Error";

/// App 更新信息
interface AppUpdateInfo {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  release_notes: string;
  download_url: string;
}

/// easytier-core 更新信息
interface CoreUpdateInfo {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  download_url: string;
}

/// 下载进度
interface DownloadProgress {
  downloaded: number;
  total: number;
  percentage: number;
}

/// 最短加载延迟（ms）
const MIN_LOADING_DELAY = 800;

/// 设置页面
export default function SettingsPage() {
  // ========== App 版本相关状态 ==========
  const [appVersion, setAppVersion] = useState<string>("");
  const [appUpdate, setAppUpdate] = useState<AppUpdateInfo | null>(null);
  const [appChecking, setAppChecking] = useState(false);
  const [appError, setAppError] = useState<string | null>(null);

  // 下载相关
  const [appDownloading, setAppDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);
  const [downloadDone, setDownloadDone] = useState(false); // 下载完成，等待用户点重启

  // ========== Core 版本相关状态 ==========
  const [coreVersion, setCoreVersion] = useState<string>("");
  const [coreUpdate, setCoreUpdate] = useState<CoreUpdateInfo | null>(null);
  const [coreChecking, setCoreChecking] = useState(false);
  const [coreError, setCoreError] = useState<string | null>(null);

  /// 加载版本信息
  const loadVersions = async () => {
    try {
      const ver = await invoke<string>("get_app_version");
      setAppVersion(ver);
    } catch {
      setAppVersion("未知");
    }
    try {
      const ver = await invoke<string>("get_core_version");
      setCoreVersion(ver);
    } catch {
      setCoreVersion("未知");
    }
  };

  /// 监听下载进度事件
  useEffect(() => {
    const unlisten = listen<DownloadProgress>(
      "update-download-progress",
      (event) => {
        setDownloadProgress(event.payload);
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  /// 检查 App 自更新
  const handleCheckAppUpdate = async () => {
    setAppChecking(true);
    setAppError(null);
    setAppUpdate(null);
    setDownloadDone(false);
    setDownloadProgress(null);
    try {
      const minDelay = new Promise<void>((r) => setTimeout(r, MIN_LOADING_DELAY));
      const [result] = await Promise.all([
        invoke<AppUpdateInfo>("check_app_update"),
        minDelay,
      ]);
      setAppUpdate(result);
    } catch (e) {
      const msg = typeof e === "string" ? e : (e as Error).message || "未知错误";
      setAppError(msg);
    } finally {
      setAppChecking(false);
    }
  };

  /// 下载更新（带进度）
  const handleDownloadUpdate = async () => {
    if (!appUpdate?.download_url) return;
    setAppDownloading(true);
    setAppError(null);
    setDownloadProgress({ downloaded: 0, total: 0, percentage: 0 });
    try {
      // 最小延迟确保进度条可见（即使下载很快/缓存命中）
      const minDelay = new Promise<void>((r) => setTimeout(r, MIN_LOADING_DELAY));
      const [,] = await Promise.all([
        invoke("download_app_update", { downloadUrl: appUpdate.download_url }),
        minDelay,
      ]);
      setDownloadDone(true);
    } catch (e) {
      const msg = typeof e === "string" ? e : (e as Error).message || "未知错误";
      setAppError(`下载更新失败：${msg}`);
    } finally {
      setAppDownloading(false);
    }
  };

  /// 立即重启安装更新
  const handleRestartInstall = async () => {
    setAppError(null);
    try {
      await invoke("install_app_update");
      // 成功后 App 会退出重启，不会执行到这里
    } catch (e) {
      const msg = typeof e === "string" ? e : (e as Error).message || "未知错误";
      setAppError(`安装更新失败：${msg}`);
    }
  };

  /// 检查 easytier-core 更新
  const handleCheckCoreUpdate = async () => {
    setCoreChecking(true);
    setCoreError(null);
    setCoreUpdate(null);
    try {
      const minDelay = new Promise<void>((r) => setTimeout(r, MIN_LOADING_DELAY));
      const [result] = await Promise.all([
        invoke<CoreUpdateInfo>("check_core_update"),
        minDelay,
      ]);
      setCoreUpdate(result);
    } catch (e) {
      const msg = typeof e === "string" ? e : (e as Error).message || "未知错误";
      setCoreError(msg);
    } finally {
      setCoreChecking(false);
    }
  };

  /// 页面初始化
  useEffect(() => {
    loadVersions();
  }, []);

  // ========== 格式化文件大小 ==========
  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  // ========== 公共组件：更新版本 Chip ==========
  const renderUpdateChip = (hasUpdate: boolean, latestVersion: string) => {
    if (hasUpdate) {
      return (
        <Chip
          label={`发现新版本 ${latestVersion}`}
          color="primary"
          size="small"
        />
      );
    }
    return (
      <Chip
        label="已是最新版本"
        color="success"
        size="small"
        variant="outlined"
      />
    );
  };

  // ========== 公共组件：错误提示条 ==========
  const renderError = (error: string | null) => {
    if (!error) return null;
    return (
      <Box
        sx={{
          mt: 2,
          p: 1.5,
          bgcolor: "error.main",
          color: "error.contrastText",
          borderRadius: 1,
          display: "flex",
          alignItems: "center",
          gap: 1,
        }}
      >
        <ErrorIcon fontSize="small" />
        <Typography variant="body2">{error}</Typography>
      </Box>
    );
  };

  // ========== 公共组件：版本卡片 ==========
  const renderVersionCard = (
    icon: React.ReactNode,
    title: string,
    version: string,
    checking: boolean,
    onCheck: () => void,
    error: string | null,
    update: AppUpdateInfo | CoreUpdateInfo | null,
    // 以下仅 App 卡片使用
    downloading?: boolean,
    progress?: DownloadProgress | null,
    isDownloadDone?: boolean,
    onDownload?: () => void,
    onRestart?: () => void
  ) => (
    <Card sx={{ mb: 2 }}>
      <CardContent>
        {/* 头部：图标 + 版本信息 + 按钮 */}
        <Box
          sx={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
            {icon}
            <Box>
              <Typography variant="subtitle2">{title}</Typography>
              <Typography variant="body2" color="text.secondary">
                {version || "加载中..."}
              </Typography>
            </Box>
          </Box>
          <Button
            variant="outlined"
            size="small"
            startIcon={
              checking ? <CircularProgress size={16} /> : <UpdateIcon />
            }
            onClick={onCheck}
            disabled={checking}
          >
            {checking ? "检查中..." : "检查更新"}
          </Button>
        </Box>

        {/* 错误提示 */}
        {renderError(error)}

        {/* 更新结果 */}
        {update && (
          <Box
            sx={{
              mt: 2,
              pt: 2,
              borderTop: "1px solid",
              borderColor: "divider",
            }}
          >
            {/* 版本信息行：版本号 + Chip + 操作按钮（右对齐） */}
            <Box
              sx={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
              }}
            >
              <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                <Typography variant="body2" sx={{ fontWeight: "medium" }}>
                  {update.current_version} →
                </Typography>
                {renderUpdateChip(update.has_update, update.latest_version)}
              </Box>

              {/* 操作按钮区（右对齐） */}
              {update.has_update && (
                <Box sx={{ display: "flex", gap: 1 }}>
                  {/* 下载完成 → 显示立即重启 */}
                  {isDownloadDone && onRestart ? (
                    <Button
                      variant="contained"
                      size="small"
                      color="success"
                      startIcon={<RestartIcon />}
                      onClick={onRestart}
                    >
                      立即重启
                    </Button>
                  ) : onDownload ? (
                    <Button
                      variant="contained"
                      size="small"
                      color="primary"
                      startIcon={
                        downloading ? (
                          <CircularProgress size={14} />
                        ) : (
                          <DownloadIcon />
                        )
                      }
                      onClick={onDownload}
                      disabled={downloading}
                    >
                      {downloading ? "下载中..." : "立即更新"}
                    </Button>
                  ) : null}
                </Box>
              )}
            </Box>

            {/* 下载进度条 */}
            {downloading && progress && (
              <Box sx={{ mt: 1.5 }}>
                <LinearProgress
                  variant={progress.total > 0 ? "determinate" : "indeterminate"}
                  value={progress.total > 0 ? progress.percentage : undefined}
                />
                <Typography
                  variant="caption"
                  color="text.secondary"
                  sx={{ mt: 0.5, display: "block" }}
                >
                  {progress.total > 0
                    ? `${formatSize(progress.downloaded)} / ${formatSize(progress.total)} (${progress.percentage}%)`
                    : "正在下载..."}
                </Typography>
              </Box>
            )}

            {/* 下载完成提示 */}
            {isDownloadDone && !downloading && (
              <Typography
                variant="body2"
                color="success.main"
                sx={{ mt: 1.5, display: "flex", alignItems: "center", gap: 0.5 }}
              >
                下载完成，点击「立即重启」开始安装更新
              </Typography>
            )}
          </Box>
        )}
      </CardContent>
    </Card>
  );

  return (
    <Box>
      <Typography variant="h5" gutterBottom sx={{ fontWeight: "bold" }}>
        设置
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        应用配置与版本管理
      </Typography>

      {/* App 版本卡片 */}
      {renderVersionCard(
        <AppIcon color="primary" />,
        "应用版本",
        appVersion,
        appChecking,
        handleCheckAppUpdate,
        appError,
        appUpdate,
        appDownloading,
        downloadProgress,
        downloadDone,
        handleDownloadUpdate,
        handleRestartInstall
      )}

      {/* easytier-core 版本卡片 */}
      {renderVersionCard(
        <CoreIcon color="action" />,
        "easytier-core",
        coreVersion,
        coreChecking,
        handleCheckCoreUpdate,
        coreError,
        coreUpdate
      )}
    </Box>
  );
}
