import { Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import ProfileEditor from "./pages/ProfileEditor";
import ActivityLogPage from "./pages/ActivityLog";
import DryRunPreview from "./pages/DryRunPreview";
import RestoreCenter from "./pages/RestoreCenter";
import SettingsPage from "./pages/Settings";
import { AppProvider } from "./hooks/useAppContext";

export default function App() {
  return (
    <AppProvider>
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/profiles" element={<ProfileEditor />} />
          <Route path="/profiles/:id" element={<ProfileEditor />} />
          <Route path="/logs" element={<ActivityLogPage />} />
          <Route path="/dry-run/:profileId" element={<DryRunPreview />} />
          <Route path="/restore" element={<RestoreCenter />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </Layout>
    </AppProvider>
  );
}
