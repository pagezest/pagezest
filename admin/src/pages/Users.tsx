import React, { useEffect } from 'react';
import { Table, Button, Group, Text, LoadingOverlay } from '@mantine/core';
import { Plus, Edit, Trash } from 'lucide-react';
import { usePosts } from '@/contexts/PostsContext';
import { useNavigate } from 'react-router-dom';

export function Users() {
  const { posts, loading, error, fetchPosts, deletePost } = usePosts();
  const navigate = useNavigate();

  useEffect(() => {
    fetchPosts();
  }, [fetchPosts]);

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
          onClick={() => navigate('/posts/new')}
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
          {posts.map((post) => (
            <Table.Tr key={post.id}>
              <Table.Td>{post.title}</Table.Td>
              <Table.Td>{post.author}</Table.Td>
              <Table.Td>{new Date(post.createdAt).toLocaleDateString()}</Table.Td>
              <Table.Td>
                <Group gap="xs">
                  <Button
                    variant="light"
                    size="xs"
                    onClick={() => navigate(`/posts/${post.id}/edit`)}
                  >
                    <Edit size={16} />
                  </Button>
                  <Button
                    color="red"
                    variant="light"
                    size="xs"
                    onClick={() => deletePost(post.id)}
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
