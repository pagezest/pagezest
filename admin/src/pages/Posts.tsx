import React, { useEffect } from 'react';
import { Table, Button, Group, Text, LoadingOverlay } from '@mantine/core';
import { Plus, Edit, Trash, Eye } from 'lucide-react';
import { usePosts } from '../contexts/PostsContext';
import { useNavigate } from 'react-router-dom';
import { Post } from '@/types';

export function Posts() {
  const { posts, loading, error, fetchPosts, deletePost } = usePosts();
  const navigate = useNavigate();

  useEffect(() => {
    fetchPosts();
  }, [fetchPosts]);

  if (error) {
    return <Text color="red">{error}</Text>;
  }

  function previewPost(post: Post) {
    const postUrl = `/api/preview/${post.slug}`;
    window.open(postUrl);
  }

  return (
    <div className="relative">
      <LoadingOverlay visible={loading} />
      
      <Group justify="space-between" mb="lg">
        <h1 className="text-2xl font-bold">Posts</h1>
        <Button
          leftSection={<Plus size={20} />}
          onClick={() => navigate('/posts/new')}
        >
          New Post
        </Button>
      </Group>

      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Title</Table.Th>
            <Table.Th>Author</Table.Th>
            <Table.Th>Slug</Table.Th>
            <Table.Th>Created At</Table.Th>
            <Table.Th>Actions</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {posts.map((post) => (
            <Table.Tr key={post.id}>
              <Table.Td>{post.title}</Table.Td>
              <Table.Td>{post.author}</Table.Td>
              <Table.Td>/{post.slug}</Table.Td>
              <Table.Td>{new Date(post.created_at).toLocaleDateString()}</Table.Td>
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
                  <Button
                    color="green"
                    variant="light"
                    size="xs"
                    onClick={() => previewPost(post)}
                  >
                    <Eye size={16} />
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
