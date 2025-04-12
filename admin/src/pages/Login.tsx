import React from 'react';
import { TextInput, PasswordInput, Button, Paper, Title } from '@mantine/core';
import { useForm } from '@mantine/form';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';

export function Login() {
  const navigate = useNavigate();
  const { login, loading, error } = useAuth();

  const form = useForm({
    initialValues: {
      email: '',
      password: '',
    },
    validate: {
      email: (value) => (/^\S+@\S+$/.test(value) ? null : 'Invalid email'),
      password: (value) => (value.length < 6 ? 'Password must be at least 6 characters' : null),
    },
  });

  const handleSubmit = async (values: { email: string; password: string }) => {
    try {
      await login(values.email, values.password);
      navigate('/');
    } catch (error) {
      console.error('Login failed:', error);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100">
      <Paper radius="md" p="xl" className="w-full max-w-md">
        <Title order={2} className="text-center mb-6">Welcome to CMS</Title>

        <form onSubmit={form.onSubmit(handleSubmit)}>
          <TextInput
            label="Email"
            placeholder="your@email.com"
            required
            className="mb-4"
            {...form.getInputProps('email')}
          />

          <PasswordInput
            label="Password"
            placeholder="Your password"
            required
            className="mb-6"
            {...form.getInputProps('password')}
          />

          {error && (
            <div className="text-red-500 text-sm mb-4">{error}</div>
          )}

          <Button
            type="submit"
            fullWidth
            loading={loading}
          >
            Sign in
          </Button>
        </form>
      </Paper>
    </div>
  );
}