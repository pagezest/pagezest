import { Theme } from '../types';

const API_HEADERS = {
  'Accept': 'application/json',
  'Content-Type': 'application/json',
};

export const getThemes = async (): Promise<Theme[]> => {
  return fetch('/api/themes', {
    headers: {
      ...API_HEADERS,
    },
  }).then(a => a.json()).then(a => a.data);

};

export const getTheme = async (id: string): Promise<Theme | null> => {
  return fetch(`/api/theme/${id}`, {
    headers: {
      ...API_HEADERS,
    },
  })
  .then(a => a.json())
  .then(a => a.data || []);
};

export const createTheme = async (theme: Omit<Theme, 'id' | 'created_at' | 'updated_at'>): Promise<Theme> => {
  const newTheme: Partial<Theme> = {
    ...theme,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  return fetch('/api/theme/new', {
    method: 'POST',
    headers: {
      ...API_HEADERS,
    },
    body: JSON.stringify(newTheme)
  }).then(a => a.json());
};

export const updateTheme = async (id: string, theme: Partial<Theme>): Promise<Theme> => {
  const newTheme: Partial<Theme> = {
    ...theme,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  return fetch(`/api/theme/update`, {
    method: 'POST',
    headers: {
      ...API_HEADERS,
    },
    body: JSON.stringify(newTheme)
  }).then(a => a.json());

};

export const deleteTheme = async (id: string): Promise<void> => {
  return fetch(`/api/delete`, {
    method: 'POST',
    body: `id=${id}`,
  }).then(a => a.text());
};
