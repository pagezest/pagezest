import {
  Container,
  TextInput,
  Select,
  Button,
  Stack,
  Title,
  Flex,
} from "@mantine/core";
export default function SiteSettings() {
  return (
    <Container size="md" pt="xl">
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
          <Flex justify="end" mt="md">
            <Button type="submit">Save Settings</Button>
          </Flex>
        </Stack>
      </form>
    </Container>
  );
}
