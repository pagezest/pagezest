import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import { Layout } from '@/components/Layout';
import { PrivateRoute } from '@/components/PrivateRoute';
import { Dashboard } from '@/pages/Dashboard';
import { Posts } from '@/pages/Posts';
import { PostForm } from '@/pages/PostForm';
import { Login } from '@/pages/Login';
import { Users } from '@/pages/Users';
import Settings from '@/pages/Settings';

const APP_BASENAME: string = import.meta.env.VITE_ROUTER_BASE_NAME || '/pz-admin';

export default function AppRoutes() {
  return (
    <Router basename={APP_BASENAME}>
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
