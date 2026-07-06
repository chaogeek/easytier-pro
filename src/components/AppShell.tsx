import { useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import {
  Box,
  Drawer,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  AppBar,
  Toolbar,
  Typography,
  useTheme,
} from "@mui/material";
import LinkIcon from "@mui/icons-material/Link";
import HubIcon from "@mui/icons-material/Hub";
import SettingsIcon from "@mui/icons-material/Settings";

// 侧边栏宽度
const DRAWER_WIDTH = 200;

// 导航菜单配置
const NAV_ITEMS = [
  {
    path: "/connections",
    label: "连接",
    icon: <LinkIcon />,
    shortcut: "⌘1",
  },
  {
    path: "/nodes",
    label: "节点",
    icon: <HubIcon />,
    shortcut: "⌘2",
  },
  {
    path: "/settings",
    label: "设置",
    icon: <SettingsIcon />,
    shortcut: "⌘3",
  },
];

interface AppShellProps {
  children: React.ReactNode;
}

/// 应用主布局：左侧固定侧边栏 + 右侧内容区
export default function AppShell({ children }: AppShellProps) {
  const navigate = useNavigate();
  const location = useLocation();
  const theme = useTheme();

  // 键盘快捷键切换页面
  useState(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        if (e.key === "1") navigate("/connections");
        if (e.key === "2") navigate("/nodes");
        if (e.key === "3") navigate("/settings");
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  });

  return (
    <Box sx={{ display: "flex", height: "100vh" }}>
      {/* 左侧边栏 */}
      <Drawer
        variant="permanent"
        sx={{
          width: DRAWER_WIDTH,
          flexShrink: 0,
          "& .MuiDrawer-paper": {
            width: DRAWER_WIDTH,
            boxSizing: "border-box",
            borderRight: `1px solid ${theme.palette.divider}`,
            backgroundColor: theme.palette.background.default,
          },
        }}
      >
        {/* 标题区 */}
        <Box
          sx={{
            height: 64,
            display: "flex",
            alignItems: "center",
            px: 2,
            borderBottom: `1px solid ${theme.palette.divider}`,
          }}
        >
          <Typography variant="h6" sx={{ fontWeight: "bold" }} noWrap>
            EasyTier
          </Typography>
        </Box>

        {/* 导航菜单 */}
        <List sx={{ pt: 1 }}>
          {NAV_ITEMS.map((item) => {
            const isActive = location.pathname === item.path;
            return (
              <ListItemButton
                key={item.path}
                selected={isActive}
                onClick={() => navigate(item.path)}
                sx={{
                  mx: 1,
                  borderRadius: 1,
                  mb: 0.5,
                  "&.Mui-selected": {
                    backgroundColor: theme.palette.action.selected,
                  },
                }}
              >
                <ListItemIcon
                  sx={{
                    minWidth: 36,
                    color: isActive
                      ? theme.palette.primary.main
                      : theme.palette.text.secondary,
                  }}
                >
                  {item.icon}
                </ListItemIcon>
                <ListItemText
                  primary={item.label}
                  sx={{
                    "& .MuiListItemText-primary": {
                      fontSize: 14,
                      fontWeight: isActive ? 600 : 400,
                    },
                  }}
                />
                <Typography
                  variant="caption"
                  sx={{
                    color: theme.palette.text.disabled,
                    fontSize: 11,
                  }}
                >
                  {item.shortcut}
                </Typography>
              </ListItemButton>
            );
          })}
        </List>
      </Drawer>

      {/* 右侧内容区 */}
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          display: "flex",
          flexDirection: "column",
          height: "100vh",
          overflow: "hidden",
        }}
      >
        {/* 顶部状态栏（预留） */}
        <AppBar
          position="static"
          elevation={0}
          sx={{
            backgroundColor: theme.palette.background.default,
            borderBottom: `1px solid ${theme.palette.divider}`,
          }}
        >
          <Toolbar variant="dense" sx={{ minHeight: 48 }}>
            <Typography variant="body2" color="text.secondary">
              未连接
            </Typography>
          </Toolbar>
        </AppBar>

        {/* 页面内容 */}
        <Box
          sx={{
            flexGrow: 1,
            overflow: "auto",
            p: 3,
          }}
        >
          {children}
        </Box>
      </Box>
    </Box>
  );
}
