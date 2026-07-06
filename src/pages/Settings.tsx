import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Typography,
  Box,
  Card,
  CardContent,
  Button,
  CircularProgress,
  Chip,
  Divider,
} from "@mui/material";
import UpdateIcon from "@mui/icons-material/SystemUpdateAlt";
import InfoIcon from "@mui/icons-material/Info";

/// 更新信息类型
interface AppUpdateInfo {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  release_notes: string;
  download_url: string;
}

interface CoreUpdateInfo {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  download_url: string;
}

/// 设置页面
export default function SettingsPage() {
  const [appVersion, setAppVersion] = useState<string>("");
  const [appUpdate, setAppUpdate] = useState<AppUpdateInfo | null>(null);
  const [coreUpdate, setCoreUpdate] = useState<CoreUpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);

  /// 加载版本信息
  const loadVersion = async () => {
    try {
      const ver = await invoke<string>("get_app_version");
      setAppVersion(ver);
    } catch {
      setAppVersion("未知");
    }
  };

  /// 检查更新（App + Core）
  const handleCheckUpdate = async () => {
    setChecking(true);
    try {
      const [appResult, coreResult] = await Promise.all([
        invoke<AppUpdateInfo>("check_app_update"),
        invoke<CoreUpdateInfo>("check_core_update"),
      ]);
      setAppUpdate(appResult);
      setCoreUpdate(coreResult);
    } catch (e) {
      console.error("检查更新失败:", e);
    } finally {
      setChecking(false);
    }
  };

  // 页面初始化
  useState(() => {
    loadVersion();
  });

  return (
    <Box>
      <Typography variant="h5" gutterBottom sx={{ fontWeight: "bold" }}>
        设置
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        应用配置与版本管理
      </Typography>

      {/* 版本信息卡片 */}
      <Card sx={{ mb: 2 }}>
        <CardContent>
          <Box
            sx={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
            }}
          >
            <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
              <InfoIcon color="action" />
              <Box>
                <Typography variant="subtitle2">应用版本</Typography>
                <Typography variant="body2" color="text.secondary">
                  {appVersion || "加载中..."}
                </Typography>
              </Box>
            </Box>
            <Button
              variant="outlined"
              size="small"
              startIcon={
                checking ? <CircularProgress size={16} /> : <UpdateIcon />
              }
              onClick={handleCheckUpdate}
              disabled={checking}
            >
              {checking ? "检查中..." : "检查更新"}
            </Button>
          </Box>

          {/* App 更新结果 */}
          {appUpdate && (
            <>
              <Divider sx={{ my: 2 }} />
              <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                <Typography variant="body2" sx={{ fontWeight: "medium" }}>
                  App 更新：
                </Typography>
                {appUpdate.has_update ? (
                  <Chip
                    label={`发现新版本 ${appUpdate.latest_version}`}
                    color="primary"
                    size="small"
                  />
                ) : (
                  <Chip
                    label="已是最新版本"
                    color="success"
                    size="small"
                    variant="outlined"
                  />
                )}
              </Box>
            </>
          )}

          {/* Core 更新结果 */}
          {coreUpdate && (
            <Box sx={{ display: "flex", alignItems: "center", gap: 1, mt: 1 }}>
              <Typography variant="body2" sx={{ fontWeight: "medium" }}>
                easytier-core：
              </Typography>
              {coreUpdate.has_update ? (
                <Chip
                  label={`发现新版本 ${coreUpdate.latest_version}`}
                  color="warning"
                  size="small"
                />
              ) : (
                <Chip
                  label="已是最新版本"
                  color="success"
                  size="small"
                  variant="outlined"
                />
              )}
            </Box>
          )}
        </CardContent>
      </Card>
    </Box>
  );
}
