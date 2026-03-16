import { Blocks, ExternalLink, Sparkles } from 'lucide-react';

export default function Ecosystem() {
    return (
        <section id="ecosystem" style={styles.section}>
            <div className="container" style={styles.container}>
                <div className="glass-card ecosystem-card">
                    <div style={styles.copy}>
                        <div style={styles.eyebrow}>
                            <Sparkles size={16} />
                            Ecosystem
                        </div>
                        <h2 style={styles.title}>
                            Mnemix pairs with <span className="text-gradient">mnemix-context</span>
                        </h2>
                        <p style={styles.body}>
                            For config-driven, multi-platform generation of AI coding resources,
                            use <a href="https://github.com/micahcourey/mnemix-context" target="_blank" rel="noreferrer" style={styles.link}>mnemix-context</a>.
                            It generates pre-configured AI agents, auto-activating skills, and platform adapters tailored to your entire repository.
                        </p>
                        <p style={styles.body}>
                            Mnemix Context dynamically integrates with GitHub Copilot, Cursor, Claude Code, Cline, and Windsurf, acting as the bridge between your local memory engine and your favorite AI assistant.
                        </p>
                        <a
                            href="https://github.com/micahcourey/mnemix-context"
                            target="_blank"
                            rel="noreferrer"
                            className="btn btn-secondary"
                            style={styles.button}
                        >
                            <Blocks size={18} />
                            Explore mnemix-context
                            <ExternalLink size={16} />
                        </a>
                    </div>

                    <div className="ecosystem-meta">
                        <div style={styles.metaBadge}>Companion project</div>
                        <ul style={styles.list}>
                            <li style={styles.listItem}>Auto-generates context files by parsing your codebase schema and architecture</li>
                            <li style={styles.listItem}>Pre-configured Architect, Engineer, Reviewer, and Documentation agent personas</li>
                            <li style={styles.listItem}>Universal compatibility with native adapters for 7 leading AI tools</li>
                        </ul>
                    </div>
                </div>
            </div>
        </section>
    );
}

const styles = {
    section: {
        padding: '2rem 0 8rem',
    },
    container: {
        display: 'flex',
        flexDirection: 'column' as const,
    },
    copy: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '1rem',
    },
    eyebrow: {
        display: 'inline-flex',
        alignItems: 'center',
        gap: '0.5rem',
        color: 'var(--color-primary)',
        fontSize: '0.9rem',
        fontWeight: 600,
        textTransform: 'uppercase' as const,
        letterSpacing: '0.08em',
    },
    title: {
        marginBottom: '0.25rem',
    },
    body: {
        color: 'var(--color-text-muted)',
        fontSize: '1.05rem',
        lineHeight: 1.7,
        maxWidth: '56ch',
    },
    link: {
        color: 'var(--color-primary)',
        textDecoration: 'none',
    },
    button: {
        marginTop: '0.5rem',
        width: 'fit-content',
    },
    metaBadge: {
        color: 'var(--color-primary)',
        fontSize: '0.85rem',
        fontWeight: 600,
        textTransform: 'uppercase' as const,
        letterSpacing: '0.08em',
    },
    list: {
        margin: 0,
        paddingLeft: '1.25rem',
        color: 'var(--color-text)',
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '0.75rem',
    },
    listItem: {
        lineHeight: 1.6,
    },
};
