import { Post } from '../types';

const API_HEADERS = {
  'Accept': 'application/json',
  'Content-Type': 'application/json',
};

export const getPosts = async (): Promise<Post[]> => {
  return fetch('/api/blogs', {
    headers: {
      ...API_HEADERS,
    },
  }).then(a => a.json()).then(a => a.data);

};

export const getPost = async (id: string): Promise<Post | null> => {
  return fetch(`/api/blog/${id}`, {
    headers: {
      ...API_HEADERS,
    },
  }).then(a => a.json()).then(a => a.data);
};

export const createPost = async (post: Omit<Post, 'id' | 'created_at' | 'updated_at'>): Promise<Post> => {
  const newPost: Partial<Post> = {
    ...post,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  return fetch('/api/blog/new', {
    method: 'POST',
    headers: {
      ...API_HEADERS,
    },
    body: JSON.stringify(newPost)
  }).then(a => a.json());
};

export const updatePost = async (id: string, post: Partial<Post>): Promise<Post> => {
  const newPost: Partial<Post> = {
    ...post,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  return fetch(`/api/blog/update`, {
    method: 'POST',
    headers: {
      ...API_HEADERS,
    },
    body: JSON.stringify(newPost)
  }).then(a => a.json());

};

export const deletePost = async (id: string): Promise<void> => {
  return fetch(`/api/delete`, {
    method: 'POST',
    body: `id=${id}`,
  }).then(a => a.text());
};
