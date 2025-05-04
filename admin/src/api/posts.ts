import { Post } from '../types';

const API_HEADERS = {
  'Accept': 'application/json',
  'Content-Type': 'application/json',
};

export const getPosts = async (): Promise<Post[]> => {
  return fetch('/api/blog', {
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
  return fetch('/api/blog', {
    method: 'POST',
    headers: {
      ...API_HEADERS,
    },
    body: JSON.stringify(newPost)
  }).then(a => a.json());
};

export const updatePost = async (id: string, post: Partial<Post>): Promise<Post> => {
  const newPost: Partial<Post> = {
    id: id,
    ...post,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  return fetch(`/api/blog/${id}`, {
    method: 'PUT',
    headers: {
      ...API_HEADERS,
    },
    body: JSON.stringify(newPost)
  }).then(a => a.json());

};

export const deletePost = async (id: string): Promise<string> => {
  return fetch(`/api/blog/${id}`, {
    method: 'DELETE',
  }).then(a => a.text());
};
