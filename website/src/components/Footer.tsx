import { Github, Linkedin } from 'lucide-react';

export default function Footer() {
    return (
        <footer style={styles.footer}>
            <div className="container" style={styles.container}>
                <div style={styles.top}>
                    <div style={styles.brand}>
                        <div style={styles.logoGroup}>
                            <img src="/icon.png" alt="Mnemix Logo" style={styles.logoImage} />
                            <span style={styles.logoText}>Mnemix</span>
                        </div>
                        <p style={styles.description}>
                            The lightweight, inspectable, local-first memory engine for AI coding agents.
                        </p>
                    </div>

                    <div style={styles.links}>
                        <div style={styles.linkCol}>
                            <h4 style={styles.colTitle}>Resources</h4>
                            <a href="https://docs.mnemix.org/guide/" style={styles.link}>Documentation</a>
                            <a href="https://docs.mnemix.org/guide/#quick-start" style={styles.link}>Quickstart</a>
                            <a href="https://docs.mnemix.org/guide/host-adapters" style={styles.link}>Examples</a>
                        </div>
                        <div style={styles.linkCol}>
                            <h4 style={styles.colTitle}>Project</h4>
                            <a href="https://github.com/micahcourey/mnemix" style={styles.link}>GitHub</a>
                            <a href="https://github.com/micahcourey/mnemix/releases" style={styles.link}>Releases</a>
                            <a href="https://github.com/micahcourey/mnemix/blob/main/LICENSE" style={styles.link}>License</a>
                        </div>
                    </div>
                </div>

                <div style={styles.bottom}>
                    <p style={styles.copyright}>
                        © {new Date().getFullYear()} Mnemix by Micah Courey. MIT License.
                    </p>
                    <div style={styles.socials}>
                        <a href="https://github.com/micahcourey/mnemix" style={styles.socialLink} aria-label="GitHub">
                            <Github size={20} />
                        </a>
                        <a href="https://www.linkedin.com/in/micahcourey/" target="_blank" rel="noreferrer" style={styles.socialLink} aria-label="LinkedIn">
                            <Linkedin size={20} />
                        </a>
                    </div>
                </div>
            </div>
        </footer>
    );
}

const styles = {
    footer: {
        padding: '4rem 0 2rem 0',
        backgroundColor: 'var(--color-bg-base)',
    },
    container: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '3rem',
    },
    top: {
        display: 'flex',
        justifyContent: 'space-between',
        flexWrap: 'wrap' as const,
        gap: '2rem',
    },
    brand: {
        maxWidth: '300px',
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '1rem',
    },
    logoGroup: {
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
    },
    logoImage: {
        width: '32px',
        height: '32px',
        borderRadius: '4px',
        objectFit: 'cover' as const,
    },
    logoText: {
        fontSize: '1.25rem',
        fontWeight: 600,
        color: '#fff',
    },
    description: {
        color: 'var(--color-text-muted)',
        fontSize: '0.9rem',
        lineHeight: 1.6,
    },
    links: {
        display: 'flex',
        gap: '4rem',
    },
    linkCol: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '1rem',
    },
    colTitle: {
        fontSize: '1rem',
        fontWeight: 600,
        color: 'var(--color-text-base)',
        marginBottom: '0.25rem',
    },
    link: {
        color: 'var(--color-text-muted)',
        fontSize: '0.9rem',
        transition: 'color var(--transition-fast)',
    },
    bottom: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingTop: '2rem',
        borderTop: '1px solid var(--color-border)',
        flexWrap: 'wrap' as const,
        gap: '1rem',
    },
    copyright: {
        color: 'var(--color-text-subtle)',
        fontSize: '0.85rem',
    },
    socials: {
        display: 'flex',
        gap: '1rem',
    },
    socialLink: {
        color: 'var(--color-text-muted)',
        transition: 'color var(--transition-fast)',
    },
};
