import { getPlugins } from "@/api/plugins";
import { Plugin,  } from "@/types";
import {
  Container,
  Table,
} from "@mantine/core";
import { useEffect, useState } from "react";
import UploadPlugin from "./UploadPlugin";
import { Check } from "lucide-react";
import { useParams } from "react-router-dom";
export default function PluginsSettings() {
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [error, setError] = useState<Error|null>(null);

  useEffect(() => {
    fetchPlugins();
  }, []);

  if(true) return (<h1>
    Work in progress
  </h1>);

  async function fetchPlugins() {
    try {
      const plugins = await getPlugins();
      setPlugins(plugins.map(a => a));
    } catch(e) {
      setError(e as Error);      
      console.warn(e);
    }
  }

  return (
    <Container size="xl" pt="xl">
      <UploadPlugin />
      <Table striped>
        <Table.Thead>
          <Table.Tr>
          <Table.Th>Name</Table.Th>
          <Table.Th>Version</Table.Th>
          <Table.Th>Active</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
        {
          plugins.map(p => (<Table.Tr key={p.id}>
            <Table.Td flex={1}>{p.name}</Table.Td>
            <Table.Td flex={1}>{p.version}</Table.Td>
            <Table.Td flex={1}>{p.active ? <Check /> : null}</Table.Td>
          </Table.Tr>))
        }
        </Table.Tbody>
      </Table>
    </Container>
  );
}
