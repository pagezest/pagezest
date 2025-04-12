import {
  TextInput,
  Select,
  Button,
  Title,
  Stack,
  Container,
} from "@mantine/core";

export default function Settings() {
  return (
    <Container size="sm" pt="xl">
      <Title order={2} mb="md">
        Settings
      </Title>
      <form>
        <Stack>
          <TextInput
            label="Site Title"
            placeholder="Enter site title"
            required
          />
          <TextInput
            label="Admin Email"
            placeholder="Enter admin email"
            type="email"
            required
          />
          <Select
            label="Timezone"
            placeholder="Select timezone"
            data={["UTC", "PST", "EST", "CET"]}
            required
          />
          <Button type="submit">Save Settings</Button>
        </Stack>
      </form>
    </Container>
  );
}
