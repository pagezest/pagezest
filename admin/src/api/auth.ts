import { User } from '../types';

const MOCK_USER: User = {
  id: '1',
  name: 'John Doe',
  email: 'john@example.com',
  role: 'admin',
};

export const login = async (email: string, password: string): Promise<User> => {
  await new Promise(resolve => setTimeout(resolve, 1000));
  // In a real app, validate credentials here
  return MOCK_USER;
};

export const getCurrentUser = async (): Promise<User | null> => {
  await new Promise(resolve => setTimeout(resolve, 500));
  return MOCK_USER;
};

export const logout = async (): Promise<void> => {
  await new Promise(resolve => setTimeout(resolve, 500));
  // Clear session/token logic would go here
};