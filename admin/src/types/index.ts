import { TokensList } from "marked";

export interface PostContentType {
  md: string,
  json?: TokensList;
}

export interface Post {
  id: string;
  title: string;
  content: PostContentType;
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

export interface Plugin {
  id: string;
  name: string;
  version: string;
  manifest: PluginManifest;
}

export interface PluginManifest {
  name: string;
  version: string;
}

export interface ThemeLayout {
  sections: Record<string, Section>;
  order: string[];
}

export interface Theme {
  id: string;
  name: string;
  version: string;
  layouts: ThemeLayout[];
}

export type SettingValue = boolean | string | number;

export interface Section {
  type: string;
  blocks?: Record<string, Block>;
  block_order?: string[];
  settings?: Record<string, SettingValue>;
}

export interface Block {
  type: string;
  settings: Record<string, SettingValue>;
}

export interface ServerStats {
  num_posts: number;
  memory: number;
}
