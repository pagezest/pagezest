import { Grid, Card, Text, Group } from '@mantine/core';
import { FileText, Users, Activity } from 'lucide-react';

export function Dashboard() {
  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Dashboard</h1>
      
      <Grid>
        <Grid.Col span={{ base: 12, md: 4 }}>
          <Card shadow="sm" padding="lg">
            <Group>
              <FileText size={24} />
              <div>
                <Text size="xl" fw={700}>12</Text>
                <Text size="sm" c="dimmed">Total Posts</Text>
              </div>
            </Group>
          </Card>
        </Grid.Col>

        <Grid.Col span={{ base: 12, md: 4 }}>
          <Card shadow="sm" padding="lg">
            <Group>
              <Users size={24} />
              <div>
                <Text size="xl" fw={700}>45</Text>
                <Text size="sm" c="dimmed">Active Users</Text>
              </div>
            </Group>
          </Card>
        </Grid.Col>

        <Grid.Col span={{ base: 12, md: 4 }}>
          <Card shadow="sm" padding="lg">
            <Group>
              <Activity size={24} />
              <div>
                <Text size="xl" fw={700}>89%</Text>
                <Text size="sm" c="dimmed">System Health</Text>
              </div>
            </Group>
          </Card>
        </Grid.Col>
      </Grid>
    </div>
  );
}
