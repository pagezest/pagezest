import { getServerStats } from '@/api/stats';
import { ServerStats } from '@/types';
import { Grid, Card, Text, Group } from '@mantine/core';
import { FileText, Users, Activity } from 'lucide-react';
import { useEffect, useState } from 'react';

function formatBytes(bytes: number, decimals: number = 2): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

export function Dashboard() {
  const [loading, setLoading] = useState(true);
  const [stats, setStats] = useState<ServerStats|null>(null);
  useEffect(() => {
    setLoading(true);
    getServerStats()
    .then(resp => setStats(resp.data as ServerStats))
    .finally(() => {
      setLoading(false);
    });
  }, []);
  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Dashboard</h1>
      
      <Grid>
        <Grid.Col span={{ base: 12, md: 4 }}>
          <Card shadow="sm" padding="lg">
            <Group>
              <FileText size={24} />
              <div>
                <Text size="xl" fw={700}>{stats?.num_posts}</Text>
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
                <Text size="xl" fw={700}>0</Text>
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
                <Text size="xl" fw={700}>{stats?.memory && formatBytes(stats.memory)}</Text>
                <Text size="sm" c="dimmed">Memory</Text>
              </div>
            </Group>
          </Card>
        </Grid.Col>
      </Grid>
    </div>
  );
}
