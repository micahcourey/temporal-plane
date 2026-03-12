import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

// https://vitepress.dev/reference/site-config
export default withMermaid(defineConfig({
    title: "MNEMIX",
    description: "The Memory Engine for AI Agents",
    srcDir: './src',
    cleanUrls: true,
    appearance: 'dark', // Default to dark mode
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
                    text: 'Getting Started',
                    items: [
                        { text: 'Introduction', link: '/guide/' },
                        { text: 'Python Client SDK', link: '/guide/python' },
                        { text: 'LanceDB Rust SDK Agent Guide', link: '/guide/lancedb-rust-sdk-agent-guide' }
                    ]
                },
                {
                    text: 'Core Concepts',
                    items: [
                        { text: 'Memory Scope & Types', link: '/guide/memory-model' },
                        { text: 'Versioning & Restore', link: '/guide/versioning-and-restore' },
                        { text: 'Branch Lifecycle', link: '/guide/branch-lifecycle' },
                        { text: 'Checkpoint & Retention Policy', link: '/guide/checkpoint-and-retention-policy' },
                    ]
                }
            ],
            '/architecture/': [
                {
                    text: 'Internals',
                    items: [
                        { text: 'Architecture & Plan', link: '/architecture/mnemix-plan-v3' },
                        { text: 'Roadmap', link: '/architecture/mnemix-roadmap' }
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
