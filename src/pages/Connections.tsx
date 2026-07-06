import { Typography, Box } from "@mui/material";
import LinkIcon from "@mui/icons-material/Link";

/// 连接页面（开发中）
export default function ConnectionsPage() {
  return (
    <Box>
      <Typography variant="h5" gutterBottom sx={{ fontWeight: "bold" }}>
        连接
      </Typography>
      <Typography variant="body2" color="text.secondary">
        管理和监控当前网络连接状态
      </Typography>

      <Box
        sx={{
          mt: 4,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          minHeight: 300,
          color: "text.disabled",
        }}
      >
        <LinkIcon sx={{ fontSize: 64, mb: 2, opacity: 0.3 }} />
        <Typography variant="body1" color="text.secondary">
          暂无网络
        </Typography>
        <Typography variant="body2" color="text.disabled">
          点击「创建网络」开始使用 EasyTier 组网
        </Typography>
      </Box>
    </Box>
  );
}
