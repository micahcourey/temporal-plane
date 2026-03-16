import { Blocks, ExternalLink, Sparkles } from 'lucide-react';

export default function Ecosystem() {
    return (
        <section id="ecosystem" style={styles.section}>
            <div className="container" style={styles.container}>
                <div className="glass-card" style={styles.card}>
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
                            It complements Mnemix by generating reusable instructions, prompts,
                            skills, and coding-agent resources around your local memory workflow.
                        </p>
                        <p style={styles.body}>
                            The universal Mnemix template includes a more comprehensive coding-agent
                            adapter and agent memory policy:
                        </p>
                        <a
                            href="https://github.com/micahcourey/mnemix-context/tree/main/templates/universal/mnemix"
                            target="_blank"
                            rel="noreferrer"
                            className="btn btn-secondary"
                            style={styles.button}
                        >
                            <Blocks size={18} />
                            Explore the template
                            <ExternalLink size={16} />
                        </a>
                    </div>

                    <div style={styles.metaPanel}>
                        <div style={styles.metaBadge}>Companion project</div>
                        <ul style={styles.list}>
                            <li style={styles.listItem}>Config-driven agent resource generation</li>
                            <li style={styles.listItem}>Portable prompts, skills, and instructions</li>
                            <li style={styles.listItem}>Reusable coding-agent memory policy templates</li>
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
    card: {
        display: 'grid',
        gridTemplateColumns: '1.4fr 0.9fr',
        gap: '2rem',
        alignItems: 'stretch',
        background: 'linear-gradient(135deg, rgba(20,184,166,0.08) 0%, rgba(255,255,255,0.03) 100%)',
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
    metaPanel: {
        borderLeft: '1px solid rgba(255,255,255,0.08)',
        paddingLeft: '2rem',
        display: 'flex',
        flexDirection: 'column' as const,
        justifyContent: 'center',
        gap: '1rem',
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
