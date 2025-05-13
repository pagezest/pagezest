import React, { ChangeEventHandler, useEffect, useState } from 'react';
import { TextInput, Textarea, Button, Group, LoadingOverlay } from '@mantine/core';
import { useForm } from '@mantine/form';
import { useNavigate, useParams } from 'react-router-dom';
import { usePosts } from '../contexts/PostsContext';
import { useAuth } from '../contexts/AuthContext';
import { Post } from '../types';
import { lexer, TokensList } from 'marked';
import { getPost } from '@/api/posts';
import { buildFlatBufferFromJson } from '@/buffers/json-to-flatbuffers';

window.lexer = lexer;
export function PostForm() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { user } = useAuth();
  const { createPost, updatePost, loading } = usePosts();
  const [initialValues, setInitialValues] = useState<Partial<Post>>({
    title: '',
    slug: '',
    content: {
      md: '',
      json: undefined,
    },
    content_flatbuffer64: '',
    author: user?.name || '',
  });

  const form = useForm({
    initialValues,
    validate: {
      title: (value) => !value ? 'Title is required' : null,
      slug: (value) => /^([a-z0-9]+(-[a-z0-9]+)*)?$/.test(value) ? null : 'Invalid slug',
      content: (value) => !value ? 'Content is required' : null,
    },
  });

  useEffect(() => {
    if (id) {
      fetchPost(id);
    }
  }, [id]);

  async function fetchPost(id: string) {
    const post = await getPost(id);
    if(post) {
      if(typeof(post?.content_md) === 'string') {
        post.content = {
          md: post.content_md as string,
          json: lexer(post.content_md as string),
        };
      }
      form.setValues(post);
    }
  }

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

  function updateContent(e: ChangeEventHandler<HTMLTextAreaElement>) {
    const value = e.target.value as string;
    const json = lexer(value);
    const flatbuffer = buildFlatBufferFromJson(json);
    const flatbuffer64 = btoa(String.fromCharCode(...flatbuffer));
    form.setValues({
      content: {
        md: value,
        json,
      },
      content_flatbuffer64: flatbuffer64,
    });
  }

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
          label="Content (markdown)"
          placeholder="Enter post content"
          required
          minRows={10}
          rows={8}
          className="mb-4"
          {...form.getInputProps('content.md')}
          onChange={updateContent}
        />

        <TextInput
          label="Slug"
          placeholder="Enter post Slug"
          className="mb-4"
          {...form.getInputProps('slug')}
          pattern="^([a-z0-9]+(-[a-z0-9]+)*)?$"
          leftSection={"/"}
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
