import React from 'react';
import { AppShell, Burger, Group } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { Outlet, Link } from 'react-router-dom';
import { Home, FileText, Users, Settings } from 'lucide-react';

export function Layout() {
  const [opened, { toggle }] = useDisclosure();

  return (
    <AppShell
      header={{ height: 60 }}
      navbar={{
        width: 300,
        breakpoint: 'sm',
        collapsed: { mobile: !opened },
      }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md">
          <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
          <Group>
            <Home size={24} />
            <h1 className="text-xl font-bold">Pagezest Dashboard</h1>
          </Group>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="md">
        <Link to="/" className="flex items-center p-2 hover:bg-gray-100 rounded">
          <Home className="mr-2" size={20} />
          Dashboard
        </Link>
        <Link to="/posts" className="flex items-center p-2 hover:bg-gray-100 rounded">
          <FileText className="mr-2" size={20} />
          Posts
        </Link>
        <Link to="/users" className="flex items-center p-2 hover:bg-gray-100 rounded">
          <Users className="mr-2" size={20} />
          Users
        </Link>
        <Link to="/settings" className="flex items-center p-2 hover:bg-gray-100 rounded">
          <Settings className="mr-2" size={20} />
          Settings
        </Link>
      </AppShell.Navbar>

      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  );
}
