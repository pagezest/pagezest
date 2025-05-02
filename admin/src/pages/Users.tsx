import { useEffect } from 'react';
import { Table, Button, Group, Text, LoadingOverlay } from '@mantine/core';
import { Plus, Edit, Trash } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';

export function Users() {
  const { listUsers, deleteUser, users, loading, error, } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    listUsers();
  }, []);

  if(true) return (<h1>
    Work in progress
  </h1>);

  if (error) {
    return <Text color="red">{error}</Text>;
  }

  return (
    <div className="relative">
      <LoadingOverlay visible={loading} />
      
      <Group justify="space-between" mb="lg">
        <h1 className="text-2xl font-bold">Users</h1>
        <Button
          leftSection={<Plus size={20} />}
          onClick={() => navigate('/users/new')}
        >
          New User
        </Button>
      </Group>

      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Username</Table.Th>
            <Table.Th>Role</Table.Th>
            <Table.Th>Created At</Table.Th>
            <Table.Th>Actions</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {users.map((user) => (
            <Table.Tr key={user.id}>
              <Table.Td>{user.name}</Table.Td>
              <Table.Td>{user.role}</Table.Td>
              <Table.Td>{new Date(user.createdAt).toLocaleDateString()}</Table.Td>
              <Table.Td>
                <Group gap="xs">
                  <Button
                    variant="light"
                    size="xs"
                    onClick={() => navigate(`/users/${user.id}/edit`)}
                  >
                    <Edit size={16} />
                  </Button>
                  <Button
                    color="red"
                    variant="light"
                    size="xs"
                    onClick={() => deleteUser(user.id)}
                  >
                    <Trash size={16} />
                  </Button>
                </Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </div>
  );
}
