// @ts-check
// Docusaurus configuration for the Copilot SDK for Rust documentation site.
// See: https://docusaurus.io/docs/api/docusaurus-config

import {themes as prismThemes} from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Copilot SDK for Rust',
  tagline: 'A Rust SDK for the GitHub Copilot CLI agent runtime',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  // Production URL and base path (configured for GitHub Pages on dayour/copilot-rust-sdk).
  url: 'https://dayour.github.io',
  baseUrl: '/copilot-rust-sdk/',

  organizationName: 'dayour',
  projectName: 'copilot-rust-sdk',

  onBrokenLinks: 'throw',

  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          editUrl: 'https://github.com/dayour/copilot-rust-sdk/tree/main/website/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      image: 'img/docusaurus-social-card.jpg',
      colorMode: {
        respectPrefersColorScheme: true,
      },
      docs: {
        sidebar: {
          hideable: true,
          autoCollapseCategories: true,
        },
      },
      navbar: {
        title: 'Copilot SDK for Rust',
        logo: {
          alt: 'Copilot SDK for Rust',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docsSidebar',
            position: 'left',
            label: 'Docs',
          },
          {to: '/docs/api/client', label: 'API', position: 'left'},
          {to: '/docs/examples', label: 'Examples', position: 'left'},
          {href: 'https://docs.rs/copilot-sdk', label: 'docs.rs', position: 'right'},
          {href: 'https://crates.io/crates/copilot-sdk', label: 'crates.io', position: 'right'},
          {
            href: 'https://github.com/dayour/copilot-rust-sdk',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {label: 'Introduction', to: '/docs/intro'},
              {label: 'Installation', to: '/docs/getting-started/installation'},
              {label: 'Architecture', to: '/docs/core-concepts/architecture'},
              {label: 'API Reference', to: '/docs/api/client'},
            ],
          },
          {
            title: 'Guides',
            items: [
              {label: 'Tools', to: '/docs/guides/tools'},
              {label: 'Hooks', to: '/docs/guides/hooks'},
              {label: 'BYOK', to: '/docs/guides/byok'},
              {label: 'MCP Servers', to: '/docs/guides/mcp'},
            ],
          },
          {
            title: 'More',
            items: [
              {label: 'GitHub', href: 'https://github.com/dayour/copilot-rust-sdk'},
              {label: 'docs.rs', href: 'https://docs.rs/copilot-sdk'},
              {label: 'crates.io', href: 'https://crates.io/crates/copilot-sdk'},
              {label: 'Upstream SDKs', href: 'https://github.com/github/copilot-sdk'},
            ],
          },
        ],
        copyright: `Copyright (c) ${new Date().getFullYear()} copilot-sdk-rust contributors. Built with Docusaurus. MIT Licensed.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
        additionalLanguages: ['rust', 'toml', 'bash', 'json', 'yaml', 'powershell'],
      },
    }),
};

export default config;
