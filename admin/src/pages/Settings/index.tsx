import {
  Box,
  Tabs,
} from "@mantine/core";
import SiteSettings from "./SiteSettings";
import PluginsSettings from "./PluginsSettings";
import { useParams } from "react-router-dom";
import { useState } from "react";

export default function Settings() {
  const params = useParams();
  const [page, setPage] = useState(params.page || 'site_settings');

  return (
    <Tabs value={page} onChange={setPage}>
      <Tabs.List>
        <Tabs.Tab value="site_settings">
          Site Settings
        </Tabs.Tab>
        <Tabs.Tab value="plugins">
          Plugins
        </Tabs.Tab>
      </Tabs.List>
      <Tabs.Panel value="site_settings">
        <SiteSettings />
      </Tabs.Panel>
      <Tabs.Panel value="plugins">
        <PluginsSettings />
      </Tabs.Panel>
    </Tabs>
  );
}
