import { Hexagon, Database, History, Search, Zap, ShieldCheck } from 'lucide-react';

export default function DetailedFeatures() {
    const features = [
        {
            icon: <Database className="text-primary" />,
            title: "Local-First Persistence",
            description: "Built on LanceDB for lightning-fast, embedded storage. Your memories stay strictly on your machine."
        },
        {
            icon: <History className="text-secondary" />,
            title: "Versioned History",
            description: "Every write is immutable. Roll back the store to any point in time or audit the agent's full evolution."
        },
        {
            icon: <ShieldCheck className="text-primary" />,
            title: "Secure Isolation",
            description: "Strict workspace siloing ensures zero context leakage between unrelated codebases or projects."
        },
        {
            icon: <Zap className="text-secondary" />,
            title: "Progressive Disclosure",
            description: "Layered retrieval (Pinned, Summary, Archival) prevents flooding the LLM's context window."
        },
        {
            icon: <Search className="text-primary" />,
            title: "Hybrid Search",
            description: "Combine full-text search with metadata filtering and importance-based ranking heuristics."
        },
        {
            icon: <Hexagon className="text-secondary" />,
            title: "Typed Memory",
            description: "Organize knowledge by kind: observations, decisions, procedures, facts, and warnings."
        }
    ];

    return (
        <section id="features" style={styles.section}>
            <div className="container">
                <div style={styles.header}>
                    <h2 style={styles.title}>Engineered for <span className="text-gradient">Production Agents</span></h2>
                    <p style={styles.subtitle}>
                        Mnemix provides the plumbing needed to build durable, multi-session agentic workflows.
                    </p>
                </div>

                <div className="grid grid-cols-3" style={styles.grid}>
                    {features.map((feature, index) => (
                        <div key={index} className="glass-card feature-card animate-on-scroll" style={styles.card}>
                            <div style={styles.iconWrapper}>
                                {feature.icon}
                            </div>
                            <h3 style={styles.featureTitle}>{feature.title}</h3>
                            <p style={styles.featureDesc}>{feature.description}</p>
                        </div>
                    ))}
                </div>
            </div>
        </section>
    );
}

const styles = {
    section: {
        padding: '100px 0',
        position: 'relative' as const,
    },
    header: {
        textAlign: 'center' as const,
        marginBottom: '60px',
    },
    title: {
        marginBottom: '1rem',
    },
    subtitle: {
        fontSize: '1.2rem',
        color: 'var(--color-text-muted)',
        maxWidth: '700px',
        margin: '0 auto',
    },
    grid: {
        gap: '2rem',
    },
    card: {
        display: 'flex',
        flexDirection: 'column' as const,
        alignItems: 'flex-start',
        gap: '1rem',
        padding: '2rem',
        height: '100%',
        transition: 'transform 0.3s ease, box-shadow 0.3s ease',
    },
    iconWrapper: {
        width: '48px',
        height: '48px',
        borderRadius: '12px',
        background: 'rgba(20, 184, 166, 0.1)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        marginBottom: '0.5rem',
    },
    featureTitle: {
        fontSize: '1.25rem',
        fontWeight: 600,
        margin: 0,
    },
    featureDesc: {
        color: 'var(--color-text-muted)',
        fontSize: '1rem',
        margin: 0,
        lineHeight: 1.6,
    }
};
