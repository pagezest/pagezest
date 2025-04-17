import React from 'react';
import { AppShell, Burger, Group, Tree, TreeNodeData, } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { Outlet, Link, useLocation, useMatch, useMatches } from 'react-router-dom';
import { Home, FileText, Users, Settings, Square, Paperclip, Palette, Puzzle } from 'lucide-react';



export function Layout() {
  const location = useLocation();
  const [opened, { toggle }] = useDisclosure();

  const navbarItems: TreeNodeData[] = [{
    label: 'Dashboard',
    value: 'dashboard',
    nodeProps: {path: '/',}
  }, {
    label: 'Posts',
    value: 'posts',
    nodeProps: {path: '/posts',}
  }, {
    label: 'Settings',
    value: 'settings',
    nodeProps: {path: '/settings',},
    children: [{
      label: 'Themes',
      value: 'themes',
      nodeProps: {path: '/settings/themes',},
    }, {
      label: 'Plugins',
      value: 'plugins',
      nodeProps: {path: '/settings/plugins',},
    }],
  }, {
    label: 'Users',
    value: 'users',
    nodeProps: {path: '/users',},
  }];

  const renderNode = ({node, expanded, hasChildren, elementProps}) => { 
    let icon = <Square />;
    switch(node.value) {
      case 'dashboard':
        icon = <Home />
      break;
      case 'settings':
        icon = <Settings />
      break;
      case 'posts':
        icon = <Paperclip />
      break;
      case 'users':
        icon = <Users />
      break;
      case 'themes':
        icon = <Palette />
      break;
      case 'plugins':
        icon = <Puzzle />
      break;
      default:
        icon = <span>{node.value}</span>
    }
    return (
      <Group {...elementProps} gap={7} mb="md" c={true ? 'blue': ''}>
        {icon}
        <span>{node.label}</span>
      </Group>
    );
  };

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
        <Tree data={navbarItems}
          levelOffset={32}
          expandOnClick
          expandOnSpace
          renderNode={renderNode}>
        </Tree>
      </AppShell.Navbar>

      {/*
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
        */}

      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  );
}
