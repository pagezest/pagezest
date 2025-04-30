import { User } from '../types';

export function getUsers(): Promise<User[]> {
  return fetch('/api/users').then(a => a.json())
  .then(a => {
    if(a.data) return a.data;
    if(a.error) throw new Error(a.error);
    throw new Error('Unknown Error');
  });
}

export function deleteUser(id): Promise<voi> {
  return fetch(`/api/users/${id}`, {
    method: 'DELETE',
  })
  .then(a => a.json());
}
