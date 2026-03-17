import { Code2, Braces, Sparkles, Terminal } from 'lucide-react';

export default function HowItWorks() {
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
                        <div style={styles.windowHeader}>
                            <div style={styles.windowTab}><Terminal size={14} style={{ display: 'inline', verticalAlign: 'middle', marginRight: '6px' }} />pip install mnemix</div>
                        </div>
                        <div style={styles.windowHeader2}>
                            <div style={styles.windowTab}>agent.py</div>
                        </div>
                        <pre
                            className="mono"
                            style={styles.code}
                            dangerouslySetInnerHTML={{
                                __html: `
<span style="color: #ff7b72">from</span> mnemix <span style="color: #ff7b72">import</span> Mnemix, RememberRequest

<span style="color: #8b949e"># 1. Initialize the local store</span>
client = Mnemix(store=<span style="color: #a5d6ff">".mnemix"</span>)
client.init()

<span style="color: #8b949e"># 2. Persist a decision</span>
client.remember(RememberRequest(
  id=<span style="color: #a5d6ff">"mem-001"</span>,
  scope=<span style="color: #a5d6ff">"my-project"</span>,
  kind=<span style="color: #a5d6ff">"decision"</span>,
  title=<span style="color: #a5d6ff">"Use LanceDB for storage"</span>,
  summary=<span style="color: #a5d6ff">"Chosen for embedded Arrow."</span>,
  importance=<span style="color: #79c0ff">80</span>,
  tags=[<span style="color: #a5d6ff">"architecture"</span>]
))

<span style="color: #8b949e"># 3. Retrieve context for next session</span>
context = client.recall()
<span style="color: #ff7b72">for</span> entry <span style="color: #ff7b72">in</span> context.pinned_context:
  <span style="color: #d2a8ff">print</span>(<span style="color: #a5d6ff">f"[pinned] {entry.memory.title}"</span>)
`.trim()
                            }}
                        />
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
    },
    codeCard: {
        padding: 0,
        background: '#0d1117',
        borderColor: 'rgba(255,255,255,0.05)',
    },
    windowHeader: {
        padding: '0.75rem 1rem',
        borderBottom: '1px solid rgba(255,255,255,0.05)',
        display: 'flex',
        backgroundColor: 'rgba(20, 184, 166, 0.05)',
        alignItems: 'center',
    },
    windowHeader2: {
        padding: '0.5rem 1rem',
        borderBottom: '1px solid rgba(255,255,255,0.05)',
        display: 'flex',
    },
    windowTab: {
        fontSize: '0.85rem',
        color: 'var(--color-text-muted)',
    },
    code: {
        padding: '1.5rem',
        overflowX: 'auto' as const,
        fontSize: '0.9rem',
        margin: 0,
        lineHeight: 1.5,
    },
    keyword: { color: '#ff7b72' },
    string: { color: '#a5d6ff' },
    comment: { color: '#8b949e' },
    number: { color: '#79c0ff' },
    fn: { color: '#d2a8ff' },
};
