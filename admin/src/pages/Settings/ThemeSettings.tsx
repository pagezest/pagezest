import { Theme,  } from "@/types";
import {
  Container,
  Table,
} from "@mantine/core";
import { useEffect, useState } from "react";
import UploadTheme from "./UploadTheme";
import { Check } from "lucide-react";
import { getThemes } from "@/api/themes";
export default function ThemeSettings() {
  const [themes, setThemes] = useState<Theme[]>([]);
  const [error, setError] = useState<Error|null>(null);

  useEffect(() => {
    fetchThemes();
  }, []);

  async function fetchThemes() {
    try {
      const themes = await getThemes();
      setThemes(themes.map(a => a));
    } catch(e) {
      setError(e as Error);      
      console.warn(e);
    }
  }

  return (
    <Container size="xl" pt="xl">
      <UploadTheme />
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
          themes.map(p => (<Table.Tr key={p.id}>
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
