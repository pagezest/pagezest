import { MantineProvider } from '@mantine/core';
import { AuthProvider } from './contexts/AuthContext';
import { PostsProvider } from './contexts/PostsContext';
import AppRoutes from './routes';
import '@mantine/core/styles.css';


function App() {
  return (
    <MantineProvider>
      <AuthProvider>
        <PostsProvider>
          <AppRoutes />
        </PostsProvider>
      </AuthProvider>
    </MantineProvider>
  );
}

export default App;

export { App };
