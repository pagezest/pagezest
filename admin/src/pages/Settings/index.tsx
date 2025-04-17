import { useState } from "react";
import { useParams } from "react-router-dom";
import {
  Tabs,
} from "@mantine/core";
import { Palette, Puzzle, SettingsIcon } from "lucide-react";
import SiteSettings from "./SiteSettings";
import PluginsSettings from "./PluginsSettings";
import ThemeSettings from "./ThemeSettings";

export default function Settings() {
  const params = useParams();
  const [page, setPage] = useState(params.page || 'site_settings');

  return (
    <Tabs value={page} onChange={setPage}>
      <Tabs.List>
        <Tabs.Tab value="site_settings" leftSection={<SettingsIcon />}>
          Site Settings
        </Tabs.Tab>
        <Tabs.Tab value="themes" leftSection={<Palette />}>
          Themes
        </Tabs.Tab>
        <Tabs.Tab value="plugins" leftSection={<Puzzle />}>
          Plugins
        </Tabs.Tab>
      </Tabs.List>
      <Tabs.Panel value="site_settings">
        <SiteSettings />
      </Tabs.Panel>
      <Tabs.Panel value="themes">
        <PluginsSettings />
      </Tabs.Panel>
      <Tabs.Panel value="plugins">
        <ThemeSettings />
      </Tabs.Panel>
    </Tabs>
  );
}
