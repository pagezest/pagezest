export interface Post {
  id: string;
  title: string;
  content: string;
  author: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'editor' | 'viewer';
}
