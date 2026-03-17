import { FolderTree, Pin, History, Database, Shield, MoveLeft } from 'lucide-react';

const features = [
    {
        icon: <FolderTree size={24} />,
        title: 'Scoped Memory',
        description: 'Organize memories by project or context. Never cross-contaminate knowledge between different codebases.',
    },
    {
        icon: <Pin size={24} />,
        title: 'Pinned Context',
        description: 'Pin critical decisions or preferences to always surface first. Progressive disclosure keeps the context window clean.',
    },
    {
        icon: <Database size={24} />,
        title: 'Typed Memory',
        description: 'Distinct types like observation, decision, preference, fact, and warning help the agent reason about retrieved data.',
    },
    {
        icon: <MoveLeft size={24} />,
        title: 'Time-Travel Restore',
        description: 'Restore the store to any prior version or checkpoint as a new head state. Easily undo agent mistakes.',
    },
    {
        icon: <History size={24} />,
        title: 'Version History',
        description: 'Every write creates an immutable version. Inspect and browse the full timeline of your agent\'s thought process.',
    },
    {
        icon: <Shield size={24} />,
        title: 'Local First',
        description: 'Runs entirely on your filesystem using Arrow-native embedded storage. Zero cloud dependencies or latency.',
    },
];

export default function Features() {
    return (
        <section id="features" style={styles.section}>
            <div className="container" style={styles.container}>
                <div style={styles.header}>
                    <h2 style={styles.title}>Designed for Agents. <br />Built for <span className="text-gradient">Reliability.</span></h2>
                    <p style={styles.subtitle}>
                        Mnemix solves the "cold start" problem by providing a persistent, searchable, and versioned memory backend.
                    </p>
                </div>

                <div className="grid grid-cols-3" style={styles.grid}>
                    {features.map((feature, index) => (
                        <div key={index} className="glass-card" style={styles.card}>
                            <div style={styles.iconWrapper}>
                                {feature.icon}
                            </div>
                            <h3 style={styles.cardTitle}>{feature.title}</h3>
                            <p style={styles.cardDesc}>{feature.description}</p>
                        </div>
                    ))}
                </div>
            </div>
        </section>
    );
}

const styles = {
    section: {
        padding: '8rem 0',
        position: 'relative' as const,
    },
    container: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '4rem',
    },
    header: {
        textAlign: 'center' as const,
        maxWidth: '700px',
        margin: '0 auto',
    },
    title: {
        marginBottom: '1rem',
    },
    subtitle: {
        color: 'var(--color-text-muted)',
        fontSize: '1.2rem',
    },
    grid: {
        gap: '2rem',
    },
    card: {
        display: 'flex',
        flexDirection: 'column' as const,
        alignItems: 'flex-start' as const,
        gap: '1rem',
        background: 'linear-gradient(145deg, rgba(255,255,255,0.03) 0%, rgba(20,184,166,0.02) 100%)',
    },
    iconWrapper: {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: '48px',
        height: '48px',
        borderRadius: '12px',
        background: 'rgba(20, 184, 166, 0.1)',
        color: 'var(--color-primary)',
        marginBottom: '0.5rem',
    },
    cardTitle: {
        fontSize: '1.25rem',
        fontWeight: 600,
    },
    cardDesc: {
        color: 'var(--color-text-muted)',
        fontSize: '0.95rem',
        lineHeight: 1.6,
    },
};
