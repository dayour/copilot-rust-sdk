// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docsSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/requirements',
        'getting-started/installation',
        'getting-started/quick-start',
        'getting-started/your-first-session',
      ],
    },
    {
      type: 'category',
      label: 'Core Concepts',
      items: [
        'core-concepts/architecture',
        'core-concepts/client-lifecycle',
        'core-concepts/sessions',
        'core-concepts/events',
        'core-concepts/transport-and-protocol',
        'core-concepts/error-handling',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/messaging',
        'guides/models',
        'guides/modes',
        'guides/plans',
        'guides/agents',
        'guides/tools',
        'guides/permissions',
        'guides/user-input',
        'guides/hooks',
        'guides/infinite-sessions',
        'guides/shell',
        'guides/workspace',
        'guides/fleet',
        'guides/logging',
        'guides/attachments',
        'guides/telemetry',
        'guides/byok',
        'guides/mcp',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'api/client',
        'api/session',
        'api/types',
        'api/events',
        'api/tools',
        'api/transport',
        'api/jsonrpc',
        'api/process',
        'api/error',
      ],
    },
    'examples',
    'contributing',
    'faq',
  ],
};

export default sidebars;
