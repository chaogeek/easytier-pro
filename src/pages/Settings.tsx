import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Typography,
  Box,
  Card,
  CardContent,
  Button,
  CircularProgress,
  Chip,
} from "@mui/material";
import UpdateIcon from "@mui/icons-material/SystemUpdateAlt";
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

/// 最短加载延迟（ms），确保用户能看到动画
const MIN_LOADING_DELAY = 800;

/// 设置页面
export default function SettingsPage() {
  // ========== App 版本相关状态 ==========
  const [appVersion, setAppVersion] = useState<string>("");
  const [appUpdate, setAppUpdate] = useState<AppUpdateInfo | null>(null);
  const [appChecking, setAppChecking] = useState(false);
  const [appError, setAppError] = useState<string | null>(null);

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

  /// 检查 App 自更新
  const handleCheckAppUpdate = async () => {
    setAppChecking(true);
    setAppError(null);
    setAppUpdate(null);
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
    update: AppUpdateInfo | CoreUpdateInfo | null
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
              display: "flex",
              alignItems: "center",
              gap: 1,
            }}
          >
            <Typography variant="body2" sx={{ fontWeight: "medium" }}>
              {update.current_version} →
            </Typography>
            {renderUpdateChip(update.has_update, update.latest_version)}
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
        appUpdate
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
