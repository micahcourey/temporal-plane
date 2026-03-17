import { ArrowRight } from 'lucide-react';

export default function Hero() {
    return (
        <section style={styles.section}>
            <div style={styles.glow} />
            <div className="container grid grid-cols-2" style={styles.container}>
                {/* Left Column: Copy & Actions */}
                <div className="animate-fade-in" style={styles.content}>
                    <h1 style={styles.headline}>
                        The Local Memory Engine for <span className="text-gradient">AI Agents</span>
                    </h1>

                    <p style={styles.subtext}>
                        Mnemix gives your agent a structured, local memory store that persists between sessions. Inspect, search, and time-travel with zero cloud dependencies.
                    </p>

                    <div style={styles.actions}>
                        <a href="https://docs.mnemix.org" target="_blank" rel="noreferrer" className="btn btn-primary">
                            Get Started <ArrowRight size={18} />
                        </a>
                        <a href="https://github.com/micahcourey/mnemix" target="_blank" rel="noreferrer" className="btn btn-secondary">
                            GitHub <ArrowRight size={18} />
                        </a>
                    </div>
                </div>

                {/* Right Column: Visual Logo */}
                <div className="animate-fade-in delay-200 float hero-logo-container" style={styles.visualContainer}>
                    <img src="/logo.png" alt="Mnemix Logo" className="pulse-glow" style={styles.heroLogo} />
                </div>
            </div>
        </section>
    );
}

const styles = {
    section: {
        position: 'relative' as const,
        paddingTop: '160px',
        paddingBottom: '100px',
        overflow: 'hidden',
    },
    glow: {
        position: 'absolute' as const,
        top: '-10%',
        right: '-5%',
        width: '800px',
        height: '600px',
        background: 'radial-gradient(ellipse at center, rgba(20, 184, 166, 0.15) 0%, rgba(10, 10, 10, 0) 75%)',
        filter: 'blur(100px)',
        zIndex: -1,
        pointerEvents: 'none' as const,
    },
    container: {
        alignItems: 'center',
    },
    content: {
        display: 'flex',
        flexDirection: 'column' as const,
        alignItems: 'flex-start',
        textAlign: 'left' as const,
        gap: '1.5rem',
    },
    heroLogo: {
        width: '100%',
        maxWidth: '400px',
        height: 'auto',
        borderRadius: '50%',
        filter: 'drop-shadow(0 0 40px rgba(20, 184, 166, 0.4))',
    },
    headline: {
        marginBottom: '0.5rem',
    },
    subtext: {
        fontSize: '1.25rem',
        color: 'var(--color-text-muted)',
        maxWidth: '600px',
    },
    actions: {
        display: 'flex',
        alignItems: 'center',
        gap: '1rem',
        marginTop: '1rem',
        flexWrap: 'wrap' as const,
    },
    visualContainer: {
        width: '100%',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
    },
};
