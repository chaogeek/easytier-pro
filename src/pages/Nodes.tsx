import { Typography, Box } from "@mui/material";
import HubIcon from "@mui/icons-material/Hub";

/// 节点页面（开发中）
export default function NodesPage() {
  return (
    <Box>
      <Typography variant="h5" gutterBottom sx={{ fontWeight: "bold" }}>
        节点
      </Typography>
      <Typography variant="body2" color="text.secondary">
        查看对等节点信息，包括 IP 地址、延迟和连接类型
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
        <HubIcon sx={{ fontSize: 64, mb: 2, opacity: 0.3 }} />
        <Typography variant="body1" color="text.secondary">
          暂无节点
        </Typography>
        <Typography variant="body2" color="text.disabled">
          启动网络后，连接的节点将显示在这里
        </Typography>
      </Box>
    </Box>
  );
}
