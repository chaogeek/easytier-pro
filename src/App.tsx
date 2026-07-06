import { Routes, Route, Navigate } from "react-router-dom";
import AppShell from "./components/AppShell";
import ConnectionsPage from "./pages/Connections";
import NodesPage from "./pages/Nodes";
import SettingsPage from "./pages/Settings";

/// 应用根组件：路由配置
export default function App() {
  return (
    <AppShell>
      <Routes>
        {/* 默认跳转到连接页 */}
        <Route path="/" element={<Navigate to="/connections" replace />} />
        <Route path="/connections" element={<ConnectionsPage />} />
        <Route path="/nodes" element={<NodesPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
    </AppShell>
  );
}
