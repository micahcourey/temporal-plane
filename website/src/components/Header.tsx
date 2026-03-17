import { useState } from 'react';
import { Github, BookOpen, Menu, X } from 'lucide-react';

export default function Header() {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  return (
    <header className="header" style={styles.header}>
      <div className="container" style={styles.container}>
        <div style={styles.logoGroup}>
          <img src="/icon.png?v=2" alt="Mnemix Logo" style={styles.logoImage} />
          <span style={styles.logoText}>MNEMIX</span>
        </div>

        <nav className="desktop-nav" style={styles.nav}>
          <a href="#features" style={styles.link}>Features</a>
          <a href="#deep-dive" style={styles.link}>Architecture</a>
          <a href="#how-it-works" style={styles.link}>How it Works</a>
          <a href="https://github.com/micahcourey/mnemix" target="_blank" rel="noreferrer" style={styles.iconLink}>
            <Github size={20} />
            <span style={styles.linkText}>GitHub</span>
          </a>
          <a href="https://docs.mnemix.org/" target="_blank" rel="noreferrer" className="btn btn-secondary" style={styles.btnSecondary}>
            <BookOpen size={18} />
            Docs
          </a>
        </nav>

        <button
          className="mobile-menu-btn"
          onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
        >
          {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
        </button>
      </div>

      {isMobileMenuOpen && (
        <div className="mobile-nav-panel">
          <a href="#features" style={styles.link} onClick={() => setIsMobileMenuOpen(false)}>Features</a>
          <a href="#deep-dive" style={styles.link} onClick={() => setIsMobileMenuOpen(false)}>Architecture</a>
          <a href="#how-it-works" style={styles.link} onClick={() => setIsMobileMenuOpen(false)}>How it Works</a>
          <a href="https://github.com/micahcourey/mnemix" target="_blank" rel="noreferrer" style={styles.iconLink}>
            <Github size={20} />
            <span style={styles.linkText}>GitHub</span>
          </a>
          <a href="https://docs.mnemix.org/" target="_blank" rel="noreferrer" className="btn btn-secondary" style={{ ...styles.btnSecondary, width: 'fit-content' }}>
            <BookOpen size={18} />
            Docs
          </a>
        </div>
      )}
    </header>
  );
}

const styles = {
  header: {
    position: 'fixed' as const,
    top: 0,
    left: 0,
    right: 0,
    height: '70px',
    backgroundColor: 'rgba(10, 10, 10, 0.8)',
    backdropFilter: 'blur(12px)',
    WebkitBackdropFilter: 'blur(12px)',
    borderBottom: '1px solid var(--color-border)',
    zIndex: 50,
    display: 'flex',
    alignItems: 'center',
  },
  container: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    width: '100%',
  },
  logoGroup: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
  },
  logoImage: {
    width: '32px',
    height: '32px',
    borderRadius: '4px',
    objectFit: 'cover' as const,
  },
  logoText: {
    fontFamily: 'var(--font-cyber)',
    fontSize: '1.25rem',
    fontWeight: 700,
    letterSpacing: '0.05em',
    color: '#fff',
    textShadow: '0 0 10px rgba(20, 184, 166, 0.5)',
    textTransform: 'uppercase' as const,
  },
  nav: {
    alignItems: 'center',
    gap: '2rem',
  },
  link: {
    color: 'var(--color-text-base)',
    fontSize: '0.95rem',
    fontWeight: 500,
  },
  iconLink: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    color: 'var(--color-text-muted)',
    fontSize: '0.95rem',
    fontWeight: 500,
  },
  linkText: {
    display: 'inline-block',
  },
  btnSecondary: {
    padding: '0.5rem 1rem',
    fontSize: '0.9rem',
    gap: '0.4rem',
  }
};
