import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

// https://vitepress.dev/reference/site-config
export default withMermaid(defineConfig({
    title: "MNEMIX",
    description: "The Memory Engine for AI Agents",
    srcDir: './src',
    cleanUrls: true,
    appearance: 'dark',
    head: [
        ['link', { rel: 'icon', href: '/icon.png' }]
    ],

    themeConfig: {
        // https://vitepress.dev/reference/default-theme-config
        logo: '/icon.png',

        siteTitle: 'MNEMIX',

        search: {
            provider: 'local'
        },

        nav: [
            { text: 'Home', link: '/' },
            { text: 'Guide', link: '/guide/' },
            { text: 'Architecture', link: '/architecture/mnemix-plan-v3' }
        ],

        sidebar: {
            '/guide/': [
                {
                    text: 'Guide',
                    items: [
                        { text: 'Overview', link: '/guide/' },
                        { text: 'CLI', link: '/guide/cli' },
                        { text: 'Python', link: '/guide/python' },
                        { text: 'Host Adapters', link: '/guide/host-adapters' },
                        { text: 'Policy Runner', link: '/guide/policy-runner' },
                        { text: 'Storage Foundation', link: '/guide/lancedb' }
                    ]
                },
                {
                    text: 'Core Concepts',
                    items: [
                        { text: 'Memory Model', link: '/guide/memory-model' },
                        { text: 'Versioning & Restore', link: '/guide/versioning-and-restore' },
                        { text: 'Checkpoint & Retention Policy', link: '/guide/checkpoint-and-retention-policy' },
                        { text: 'Import Staging & Branches', link: '/guide/branch-lifecycle' }
                    ]
                }
            ],
            '/architecture/': [
                {
                    text: 'Architecture',
                    items: [
                        { text: 'Overview', link: '/architecture/mnemix-plan-v3' },
                        { text: 'Status & Roadmap', link: '/architecture/mnemix-roadmap' }
                    ]
                }
            ]
        },

        socialLinks: [
            { icon: 'github', link: 'https://github.com/micahcourey/mnemix' }
        ],

        footer: {
            message: 'Released under the MIT License.',
            copyright: 'Copyright © 2024-present Mnemix Contributors'
        }
    }
}))
