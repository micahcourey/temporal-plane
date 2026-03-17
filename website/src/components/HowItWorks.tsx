import { useState } from 'react';
import { Code2, Braces, Sparkles, Terminal, Copy } from 'lucide-react';

const examples = [
    {
        id: 'python',
        label: 'Python Adapter',
        language: 'python',
        filename: 'agent.py',
        code: `
<span style="color: #ff7b72">from</span> mnemix.adapters <span style="color: #ff7b72">import</span> ChatAdapter

<span style="color: #8b949e"># Wrap any agent with memory-persistence</span>
agent = ChatAdapter(
    model=<span style="color: #a5d6ff">"gpt-o3-mini"</span>,
    workspace=<span style="color: #a5d6ff">"project-mnemix"</span>
)

<span style="color: #8b949e"># Memories are automatically managed</span>
response = agent.chat(<span style="color: #a5d6ff">"How did we fix the CORS bug?"</span>)
<span style="color: #d2a8ff">print</span>(response)
`.trim()
    },
    {
        id: 'cli',
        label: 'CLI Tool',
        language: 'bash',
        filename: 'terminal',
        code: `
<span style="color: #8b949e"># Query memories from the terminal</span>
mnemix query <span style="color: #a5d6ff">"CORS bug"</span> --kind decision

<span style="color: #8b949e"># Time-travel to a specific checkpoint</span>
mnemix restore --version v_prev_992

<span style="color: #8b949e"># Export store for manual audit</span>
mnemix export --format json
`.trim()
    }
];

export default function HowItWorks() {
    const [activeTab, setActiveTab] = useState(0);

    return (
        <section id="how-it-works" style={styles.section}>
            <div className="container" style={styles.container}>
                <div style={styles.textCol}>
                    <h2 style={styles.title}>A unified API <br />for <span className="text-gradient">Agent Memory</span></h2>
                    <p style={styles.subtitle}>
                        Integrating Mnemix takes minutes. Use the Python client within your agent's reasoning loop to persist facts and recall context progressively.
                    </p>

                    <div style={styles.stepsList}>
                        <div style={styles.step}>
                            <div style={styles.stepIcon}><Code2 size={20} /></div>
                            <div>
                                <h4 style={styles.stepTitle}>1. Remember</h4>
                                <p style={styles.stepDesc}>Persist observations, facts, or decisions during execution.</p>
                            </div>
                        </div>
                        <div style={styles.step}>
                            <div style={styles.stepIcon}><Braces size={20} /></div>
                            <div>
                                <h4 style={styles.stepTitle}>2. Recall</h4>
                                <p style={styles.stepDesc}>Retrieve pinned context and summaries when starting a session.</p>
                            </div>
                        </div>
                        <div style={styles.step}>
                            <div style={styles.stepIcon}><Sparkles size={20} /></div>
                            <div>
                                <h4 style={styles.stepTitle}>3. Search</h4>
                                <p style={styles.stepDesc}>Perform semantic hybrid searches deep within the historical archive.</p>
                            </div>
                        </div>
                    </div>
                </div>

                <div style={styles.codeCol}>
                    <div className="glass-card" style={styles.codeCard}>
                        <div style={styles.windowHeaderStrip}>
                            <div style={styles.tabContainer}>
                                {examples.map((ex, i) => (
                                    <button
                                        key={ex.id}
                                        onClick={() => setActiveTab(i)}
                                        style={{
                                            ...styles.codeTab,
                                            color: i === activeTab ? 'var(--color-text-base)' : 'var(--color-text-subtle)',
                                            borderBottom: i === activeTab ? '2px solid var(--color-primary)' : '2px solid transparent',
                                            background: i === activeTab ? 'rgba(255,255,255,0.03)' : 'transparent',
                                        }}
                                    >
                                        {ex.label}
                                    </button>
                                ))}
                            </div>
                            <div style={styles.copyBtn}>
                                <Copy size={16} />
                            </div>
                        </div>

                        <div style={styles.codeArea}>
                            <pre
                                className="mono"
                                style={styles.code}
                                dangerouslySetInnerHTML={{
                                    __html: examples[activeTab].code
                                }}
                            />
                        </div>
                    </div>
                    <div style={styles.installBar}>
                        <Terminal size={14} style={{ marginRight: '8px' }} />
                        <span>pip install mnemix</span>
                    </div>
                </div>
            </div>
        </section>
    );
}

const styles = {
    section: {
        padding: '8rem 0',
        backgroundColor: 'var(--color-bg-surface)',
        borderTop: '1px solid var(--color-border)',
        borderBottom: '1px solid var(--color-border)',
    },
    container: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '4rem',
        alignItems: 'center',
        maxWidth: '900px',
        margin: '0 auto',
    },
    textCol: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '2rem',
        textAlign: 'center' as const,
        alignItems: 'center',
    },
    title: {
        margin: 0,
    },
    subtitle: {
        color: 'var(--color-text-muted)',
        fontSize: '1.2rem',
        maxWidth: '700px',
    },
    stepsList: {
        display: 'flex',
        gap: '2rem',
        marginTop: '1rem',
        flexWrap: 'wrap' as const,
        justifyContent: 'center',
    },
    step: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '1rem',
        alignItems: 'center',
        maxWidth: '250px',
        textAlign: 'center' as const,
    },
    stepIcon: {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: '40px',
        height: '40px',
        borderRadius: '10px',
        background: 'rgba(59, 130, 246, 0.1)',
        color: 'var(--color-secondary)',
        flexShrink: 0,
    },
    stepTitle: {
        fontSize: '1.1rem',
        marginBottom: '0.25rem',
    },
    stepDesc: {
        color: 'var(--color-text-muted)',
        fontSize: '0.9rem',
        margin: 0,
    },
    codeCol: {
        width: '100%',
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '1.5rem',
    },
    codeCard: {
        padding: 0,
        background: '#0d1117',
        borderColor: 'rgba(255,255,255,0.05)',
        borderRadius: '12px',
        overflow: 'hidden',
    },
    windowHeaderStrip: {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 1rem',
        background: 'rgba(255,255,255,0.02)',
        borderBottom: '1px solid rgba(255,255,255,0.05)',
    },
    tabContainer: {
        display: 'flex',
        gap: '1rem',
    },
    codeTab: {
        padding: '1rem 1.25rem',
        fontSize: '0.85rem',
        fontWeight: 500,
        cursor: 'pointer',
        transition: 'all 0.2s ease',
        border: 'none',
        outline: 'none',
    },
    copyBtn: {
        color: 'var(--color-text-subtle)',
        cursor: 'pointer',
        opacity: 0.6,
        '&:hover': {
            opacity: 1
        }
    },
    codeArea: {
        padding: '1.5rem',
        minHeight: '260px',
    },
    code: {
        overflowX: 'auto' as const,
        fontSize: '0.95rem',
        margin: 0,
        lineHeight: 1.6,
        fontFamily: 'var(--font-mono)',
        textAlign: 'left' as const,
    },
    installBar: {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '0.75rem 1.5rem',
        background: 'rgba(20, 184, 166, 0.05)',
        border: '1px solid rgba(20, 184, 166, 0.2)',
        borderRadius: '8px',
        color: 'var(--color-primary)',
        fontSize: '0.9rem',
        fontFamily: 'var(--font-mono)',
        alignSelf: 'center',
    },
};
