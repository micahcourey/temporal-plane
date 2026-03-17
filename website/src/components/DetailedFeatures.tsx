import { useState } from 'react';
import { Terminal, Copy, Database, ArrowRight, Layers, Lock } from 'lucide-react';

const features = [
    {
        id: "local",
        tab: "Local Store",
        title: "Autonomous Persistence",
        description: "Mnemix leverages LanceDB for a lean, embedded memory store that runs entirely on your local filesystem. High-performance columnar storage with zero infra overhead.",
        type: "diagram",
        diagram: "storage"
    },
    {
        id: "recall",
        tab: "Layered Recall",
        title: "Progressive Disclosure",
        description: "Prevents context window flooding by returning memories in three intelligent layers: pinned_context (always loads first), summaries (distilled history), and archival (full record on demand).",
        type: "diagram",
        diagram: "layers"
    },
    {
        id: "adapters",
        tab: "Host Adapters",
        title: "Workflow-Specific Policy",
        description: "Generic memory operations wrapped in host-specific adapters. Tailor the memory lifecycle for coding agents, chat assistants, or CI bots with one-line integration.",
        type: "code",
        code: `# Use the Coding Agent Adapter
from adapters import CodingAgentAdapter

adapter = CodingAgentAdapter(store=".mnemix")
context = adapter.start_task(
    scope="repo:mnemix",
    task_title="Fix build error"
)
# Prompt preamble is now ready`
    },
    {
        id: "audit",
        tab: "Timeline Audit",
        title: "Immutable History & Restore",
        description: "Every write is a discrete, immutable version. Mnemix allows you to browse the full timeline of an agent's reasoning and safely restore state to any prior checkpoint.",
        type: "code",
        code: `# Browse the thought timeline
history = client.history()
print(f"Current version: {history[0].version_id}")

# Time-travel to a previous state
client.restore(version_id="checkpoint_v2")
print("Memory state restored to checkpoint.")`
    }
];

export default function DetailedFeatures() {
    const [activeTab, setActiveTab] = useState(0);

    const handleTabClick = (index: number) => {
        setActiveTab(index);
    };

    const activeFeature = features[activeTab];

    return (
        <section id="deep-dive" style={styles.section}>
            <div className="container">
                <div style={styles.header}>
                    <h2 style={styles.sectionTitle}>The Memory Engine <span className="text-gradient">Architecture</span></h2>
                </div>

                <div className="tab-navigation" style={styles.tabsRow}>
                    {features.map((f, i) => (
                        <button
                            key={f.id}
                            onClick={() => handleTabClick(i)}
                            style={{
                                ...styles.tabBtn,
                                border: i === activeTab ? '1px solid var(--color-primary)' : '1px solid transparent',
                                backgroundColor: i === activeTab ? 'rgba(20, 184, 166, 0.05)' : 'rgba(255, 255, 255, 0.02)',
                                color: i === activeTab ? 'var(--color-primary)' : 'var(--color-text-muted)',
                            }}
                        >
                            {f.tab}
                        </button>
                    ))}
                </div>

                <div className="content-row" style={styles.contentRow}>
                    <div className="text-col" style={styles.textCol}>
                        <div key={activeFeature.id} className="animate-fade-in">
                            <h2 style={styles.title}>{activeFeature.title}</h2>
                            <p style={styles.description}>{activeFeature.description}</p>
                        </div>
                    </div>

                    <div className="visual-col" style={styles.visualCol}>
                        {activeFeature.type === 'code' ? (
                            <div className="glass-card" style={styles.codeWindow}>
                                <div style={styles.windowHeader}>
                                    <div style={styles.dots}>
                                        <div style={{ ...styles.dot, backgroundColor: '#ff5f56' }}></div>
                                        <div style={{ ...styles.dot, backgroundColor: '#ffbd2e' }}></div>
                                        <div style={{ ...styles.dot, backgroundColor: '#27c93f' }}></div>
                                    </div>
                                    <div style={styles.windowTitle}>
                                        <Terminal size={14} /> mnemix_agent.py
                                    </div>
                                    <div style={styles.copyBtn}><Copy size={14} /></div>
                                </div>
                                <div style={styles.codeArea}>
                                    <pre style={styles.pre}>
                                        <code style={styles.codeText}>{activeFeature.code}</code>
                                    </pre>
                                </div>
                            </div>
                        ) : (
                            <div className="glass-card" style={styles.diagramWindow}>
                                {activeFeature.diagram === 'storage' && (
                                    <div className="storage-diagram" style={styles.storageDiagram}>
                                        {/* Node 1 */}
                                        <div className="diagram-node" style={styles.diagramNode}>
                                            <div className="icon-wrapper" style={styles.iconWrapper}>
                                                <Terminal size={28} className="text-primary" />
                                            </div>
                                            <span className="node-label" style={styles.nodeLabel}>Your Agent</span>
                                        </div>

                                        {/* Connector 1 */}
                                        <div className="connector" style={styles.connector}>
                                            <ArrowRight size={24} className="text-muted" />
                                        </div>

                                        {/* Node 2 */}
                                        <div className="diagram-node" style={styles.diagramNode}>
                                            <div className="icon-wrapper pulse-glow" style={{ ...styles.iconWrapper, ...styles.pulseDisk }}>
                                                <Database size={28} />
                                            </div>
                                            <span className="node-label" style={styles.nodeLabel}>LanceDB (local)</span>
                                        </div>

                                        {/* Connector 2 */}
                                        <div className="connector" style={styles.connector}>
                                            <ArrowRight size={24} className="text-muted" />
                                        </div>

                                        {/* Node 3 */}
                                        <div className="diagram-node" style={styles.diagramNode}>
                                            <div className="icon-wrapper" style={styles.iconWrapper}>
                                                <Layers size={28} className="text-secondary" />
                                            </div>
                                            <span className="node-label" style={styles.nodeLabel}>Memory Records</span>
                                        </div>
                                    </div>
                                )}
                                {activeFeature.diagram === 'layers' && (
                                    <div style={styles.layersDiagram}>
                                        <div style={styles.layerCard} className="glass-card">
                                            <Lock size={16} className="text-primary" />
                                            <span>Pinned Context</span>
                                        </div>
                                        <div style={{ ...styles.layerCard, opacity: 0.8 }} className="glass-card">
                                            <Layers size={16} className="text-secondary" />
                                            <span>Summary Layer</span>
                                        </div>
                                        <div style={{ ...styles.layerCard, opacity: 0.6 }} className="glass-card">
                                            <Database size={16} className="text-muted" />
                                            <span>Archival Store</span>
                                        </div>
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </section>
    );
}

const styles = {
    section: {
        padding: '100px 0',
        backgroundColor: 'var(--color-bg-base)',
        borderTop: '1px solid var(--color-border)',
    },
    header: {
        textAlign: 'center' as const,
        marginBottom: '60px',
    },
    sectionTitle: {
        fontSize: '3rem',
        marginBottom: '1rem',
    },
    tabsRow: {
        display: 'flex',
        gap: '0.5rem',
        marginBottom: '60px',
        justifyContent: 'center',
        flexWrap: 'wrap' as const,
    },
    tabBtn: {
        padding: '0.75rem 2.5rem',
        borderRadius: '8px',
        fontSize: '0.9rem',
        fontWeight: 500,
        cursor: 'pointer',
        transition: 'all 0.2s ease',
        minWidth: '140px',
        fontFamily: 'var(--font-sans)',
    },
    contentRow: {
        display: 'flex',
        gap: '4rem',
        alignItems: 'center',
        minHeight: '400px',
    },
    textCol: {
        flex: 1,
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '1.5rem',
        textAlign: 'left' as const,
    },
    title: {
        fontSize: '2.5rem',
        lineHeight: 1.2,
        margin: 0,
    },
    description: {
        fontSize: '1.1rem',
        color: 'var(--color-text-muted)',
        lineHeight: 1.6,
        margin: 0,
    },
    progressContainer: {
        width: '100%',
        height: '2px',
        backgroundColor: 'var(--color-border)',
        marginTop: '2rem',
    },
    progressBar: {
        height: '100%',
        backgroundColor: 'var(--color-primary)',
        transition: 'width 0.05s linear',
    },
    visualCol: {
        flex: 1.2,
    },
    codeWindow: {
        padding: 0,
        background: '#0d1117',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        overflow: 'hidden',
        boxShadow: '0 25px 50px -12px rgba(0,0,0,0.5)',
    },
    diagramWindow: {
        height: '350px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(135deg, rgba(20, 184, 166, 0.05) 0%, rgba(59, 130, 246, 0.05) 100%)',
        borderRadius: '12px',
        border: '1px solid var(--color-border)',
    },
    windowHeader: {
        display: 'flex',
        alignItems: 'center',
        padding: '0.75rem 1rem',
        borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
        backgroundColor: 'rgba(255, 255, 255, 0.02)',
    },
    dots: {
        display: 'flex',
        gap: '6px',
        marginRight: '1rem',
    },
    dot: {
        width: '10px',
        height: '10px',
        borderRadius: '50%',
    },
    windowTitle: {
        fontSize: '0.85rem',
        color: 'var(--color-text-muted)',
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        flex: 1,
        justifyContent: 'center',
    },
    copyBtn: {
        color: 'var(--color-text-subtle)',
        cursor: 'pointer',
    },
    codeArea: {
        padding: '1.5rem',
        maxHeight: '350px',
        overflowY: 'auto' as const,
        textAlign: 'left' as const,
    },
    pre: {
        margin: 0,
    },
    codeText: {
        fontFamily: 'var(--font-mono)',
        fontSize: '0.95rem',
        color: '#e6edf3',
        lineHeight: 1.6,
        whiteSpace: 'pre-wrap' as const,
    },
    storageDiagram: {
        display: 'flex',
        alignItems: 'center',
        gap: '0',
        justifyContent: 'center',
    },
    diagramNode: {
        display: 'flex',
        flexDirection: 'column' as const,
        alignItems: 'center',
        gap: '1rem',
        width: '120px',
    },
    connector: {
        height: '64px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: '40px',
    },
    nodeLabel: {
        fontSize: '0.8rem',
        color: 'var(--color-text-muted)',
        fontFamily: 'var(--font-cyber)',
        letterSpacing: '0.1em',
        textAlign: 'center' as const,
        width: '100%',
    },
    iconWrapper: {
        width: '64px',
        height: '64px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        borderRadius: '12px',
        background: 'rgba(255, 255, 255, 0.03)',
        border: '1px solid var(--color-border)',
        boxSizing: 'border-box' as const,
    },
    pulseDisk: {
        borderRadius: '50%',
        border: '2px solid var(--color-primary)',
        color: 'var(--color-primary)',
    },
    arrowTable: {
        width: '60px',
        height: '60px',
        border: '1px solid var(--color-text-muted)',
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '4px',
        padding: '6px',
    },
    tableRow: {
        height: '10px',
        background: 'rgba(255,255,255,0.1)',
        width: '100%',
    },
    layersDiagram: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '1rem',
        width: '100%',
        maxWidth: '300px',
    },
    layerCard: {
        padding: '0.75rem 1rem',
        display: 'flex',
        alignItems: 'center',
        gap: '1rem',
        fontSize: '0.9rem',
    },
};
