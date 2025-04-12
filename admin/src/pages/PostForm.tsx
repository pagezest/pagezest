import React, { useEffect, useState } from 'react';
import { TextInput, Textarea, Button, Group, LoadingOverlay } from '@mantine/core';
import { useForm } from '@mantine/form';
import { useNavigate, useParams } from 'react-router-dom';
import { usePosts } from '../contexts/PostsContext';
import { useAuth } from '../contexts/AuthContext';
import { Post } from '../types';

export function PostForm() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { user } = useAuth();
  const { createPost, updatePost, loading } = usePosts();
  const [initialValues, setInitialValues] = useState<Partial<Post>>({
    title: '',
    slug: '',
    content: '',
    author: user?.name || '',
  });

  const form = useForm({
    initialValues,
    validate: {
      title: (value) => !value ? 'Title is required' : null,
      slug: (value) => !value ? 'Slug is required' : null,
      content: (value) => !value ? 'Content is required' : null,
    },
  });

  useEffect(() => {
    if (id) {
      // Fetch post data if editing
      // In a real app, you would fetch the post data here
    }
  }, [id]);

  const handleSubmit = async (values: typeof initialValues) => {
    try {
      if (id) {
        await updatePost(id, values);
      } else {
        await createPost(values as Omit<Post, 'id' | 'createdAt' | 'updatedAt'>);
      }
      navigate('/posts');
    } catch (error) {
      console.error('Failed to save post:', error);
    }
  };

  return (
    <div className="relative">
      <LoadingOverlay visible={loading} />
      
      <h1 className="text-2xl font-bold mb-6">
        {id ? 'Edit Post' : 'Create New Post'}
      </h1>

      <form onSubmit={form.onSubmit(handleSubmit)} className="max-w-2xl">
        <TextInput
          label="Title"
          placeholder="Enter post title"
          required
          className="mb-4"
          {...form.getInputProps('title')}
        />

        <Textarea
          label="Content"
          placeholder="Enter post content"
          required
          minRows={5}
          className="mb-4"
          {...form.getInputProps('content')}
        />

        <TextInput
          label="Slug"
          placeholder="Enter post Slug"
          required
          className="mb-4"
          {...form.getInputProps('slug')}
        />


        <TextInput
          label="Author"
          value={user?.name || ''}
          disabled
          className="mb-4"
        />

        <Group justify="flex-end" mt="xl">
          <Button variant="light" onClick={() => navigate('/posts')}>
            Cancel
          </Button>
          <Button type="submit">
            {id ? 'Update Post' : 'Create Post'}
          </Button>
        </Group>
      </form>
    </div>
  );
}
