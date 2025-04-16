import { Plugin } from '../types';

const API_HEADERS = {
  'Accept': 'application/json',
  'Content-Type': 'application/json',
};

export const getPlugins = async (): Promise<Plugin[]> => {
  return fetch('/api/plugins', {
    headers: {
      ...API_HEADERS,
    },
  }).then(a => a.json()).then(a => a.data);

};

export const getPlugin = async (id: string): Promise<Plugin | null> => {
  return fetch(`/api/plugin/${id}`, {
    headers: {
      ...API_HEADERS,
    },
  }).then(a => a.json()).then(a => a.data);
};

export const createPlugin = async (plugin: Omit<Plugin, 'id' | 'created_at' | 'updated_at'>): Promise<Plugin> => {
  const newPlugin: Partial<Plugin> = {
    ...plugin,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  return fetch('/api/plugin/new', {
    method: 'POST',
    headers: {
      ...API_HEADERS,
    },
    body: JSON.stringify(newPlugin)
  }).then(a => a.json());
};

export const updatePlugin = async (id: string, plugin: Partial<Plugin>): Promise<Plugin> => {
  const newPlugin: Partial<Plugin> = {
    ...plugin,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  return fetch(`/api/plugin/update`, {
    method: 'POST',
    headers: {
      ...API_HEADERS,
    },
    body: JSON.stringify(newPlugin)
  }).then(a => a.json());

};

export const deletePlugin = async (id: string): Promise<void> => {
  return fetch(`/api/delete`, {
    method: 'POST',
    body: `id=${id}`,
  }).then(a => a.text());
};
