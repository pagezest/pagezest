import { BrowserRouter as Router, Route, Navigate, Routes } from 'react-router-dom';
import { Layout } from '@/components/Layout';
import { PrivateRoute } from '@/components/PrivateRoute';
import { Dashboard } from '@/pages/Dashboard';
import { Posts } from '@/pages/Posts';
import { PostForm } from '@/pages/PostForm';
import { Login } from '@/pages/Login';
import { Users } from '@/pages/Users';
import Settings from '@/pages/Settings';

export default function AppRoutes() {
  return (
    <Router>
      <Routes>
        <Route path="/login" element={<Login />} />

        <Route path="/" element={
          <PrivateRoute>
            <Layout />
          </PrivateRoute>
          }>
          <Route index element={<Dashboard />} />
          <Route path="posts" element={<Posts />} />
          <Route path="users" element={<Users />} />
          <Route path="settings/:page" element={<Settings />} />
          <Route path="settings" element={<Settings />} />
          <Route path="posts/new" element={<PostForm />} />
          <Route path="posts/:id/edit" element={<PostForm />} />
        </Route>
      </Routes>
    </Router>

  )
}
