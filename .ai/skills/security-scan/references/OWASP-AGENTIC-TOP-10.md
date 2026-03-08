# OWASP Top 10 for Agentic Applications (2026)

Reference: [OWASP GenAI Security Project - Agentic Security Initiative](https://genai.owasp.org)

> This document provides guidance for securing AI agents that plan, decide, and act autonomously across multiple systems.

---

## Overview

The OWASP Agentic Top 10 addresses security risks specific to AI agents that operate with autonomy, tool access, and inter-agent communication capabilities. Unlike traditional LLM applications, agentic systems can compound vulnerabilities through multi-step reasoning and delegation chains.

| ID | Vulnerability | Description |
|----|--------------|-------------|
| ASI01 | Agent Goal Hijack | Manipulation of agent objectives through prompt injection or deceptive inputs |
| ASI02 | Tool Misuse & Exploitation | Legitimate tools used in unsafe or unintended ways |
| ASI03 | Identity & Privilege Abuse | Exploitation of delegation chains and inherited credentials |
| ASI04 | Agentic Supply Chain | Compromised tools, plugins, MCP servers, or agent cards |
| ASI05 | Unexpected Code Execution | Generated code bypasses security controls |
| ASI06 | Memory & Context Poisoning | Corrupted persistent memory affects future reasoning |
| ASI07 | Insecure Inter-Agent Communication | Weak authentication between agents enables spoofing |
| ASI08 | Cascading Failures | Single fault propagates across agent networks |
| ASI09 | Human-Agent Trust Exploitation | Agents manipulate users through anthropomorphism |
| ASI10 | Rogue Agents | Agents deviate from intended behavior autonomously |

---

## ASI01: Agent Goal Hijack

**Description**: Attackers manipulate an agent's objectives, task selection, or decision pathways through prompt injection, deceptive tool outputs, malicious artifacts, or poisoned external data.

### Common Attack Vectors
- Indirect prompt injection via hidden payloads in documents/web pages
- Malicious email/calendar content hijacking agent actions
- Financial agents tricked into unauthorized transfers

### Example Attacks
- **EchoLeak**: Zero-click prompt injection causing Microsoft 365 Copilot to exfiltrate data
- **Operator Prompt Injection**: Web content tricks agent into exposing private data
- **Inception Attack**: Malicious Google Doc causes ChatGPT to exfiltrate user data

### Prevention
```typescript
// ✅ Treat all natural-language inputs as untrusted
// Apply input validation before goal selection

// ✅ Require human approval for high-impact actions
if (isHighImpactAction(action)) {
  await requireHumanApproval(action);
}

// ✅ Lock and audit system prompt changes
// Changes must go through configuration management
```

---

## ASI02: Tool Misuse and Exploitation

**Description**: Agents misuse legitimate tools due to prompt injection, misalignment, or unsafe delegation - leading to data exfiltration, privilege escalation, or unintended actions.

### Common Attack Vectors
- Over-privileged tool access (email summarizer can delete/send)
- Over-scoped API access (Salesforce tool gets any record)
- Unvalidated input forwarding to shell commands
- Loop amplification causing DoS or bill spikes

### Example Attacks
- **Tool Poisoning**: Attacker corrupts MCP tool descriptors
- **EDR Bypass**: Agent chains legitimate tools to exfiltrate data
- **Approved Tool Misuse**: Ping tool used for DNS exfiltration

### Prevention
```typescript
// ✅ Least privilege for each tool
const toolPermissions = {
  emailTool: { read: true, send: false, delete: false },
  databaseTool: { select: true, delete: false }
};

// ✅ Action-level authentication
router.post('/tool/execute',
  authMiddleware.verifyToken,
  toolGatekeeper.validateToolCall,
  toolGatekeeper.applyRateLimits,
  toolExecutor.run
);

// ✅ Execution sandboxes with egress controls
```

---

## ASI03: Identity and Privilege Abuse

**Description**: Exploitation of dynamic trust and delegation in agents to escalate access by manipulating delegation chains, role inheritance, and cached credentials.

### Common Attack Vectors
- Un-scoped privilege inheritance from manager to worker agents
- Memory-based privilege retention across sessions
- Cross-agent trust exploitation (confused deputy)
- Synthetic identity injection using fake agent descriptors

### Example Attacks
- **Delegated Privilege Abuse**: Query agent inherits all finance agent permissions
- **Memory-Based Escalation**: Admin agent's cached SSH credentials reused
- **Device-Code Phishing**: Agent completes OAuth code binding to attacker

### Prevention
```typescript
// ✅ Task-scoped, time-bound permissions
const token = await issueToken({
  scope: ['read:participants'],
  audience: 'task-123',
  expiresIn: '5m'
});

// ✅ Isolate agent contexts
// Wipe state between tasks/users

// ✅ Per-action authorization
// Re-verify each privileged step with policy engine
```

---

## ASI04: Agentic Supply Chain Vulnerabilities

**Description**: Agents, tools, and artifacts from third parties may be malicious, compromised, or tampered with. Includes MCP servers, agent registries, plugins, and model weights.

### Common Attack Vectors
- Poisoned prompt templates loaded remotely
- Tool-descriptor injection in MCP/agent cards
- Typosquatted endpoints impersonating legitimate tools
- Compromised MCP registry serving malicious manifests

### Example Attacks
- **Amazon Q Compromise**: Poisoned prompt ships in VS Code extension
- **Malicious MCP Server**: npm package impersonates Postmark, BCCs emails to attacker
- **Agent-in-the-Middle**: Rogue agent card intercepts privileged traffic

### Prevention
```typescript
// ✅ Require SBOMs and AIBOMs with attestation
// ✅ Allowlist and pin dependencies
// ✅ Verify provenance before activation

// ✅ Supply chain kill switch
async function revokeCompromisedTool(toolId: string) {
  await toolRegistry.revoke(toolId);
  await deploymentService.hotDisable(toolId);
  await alertService.notifySecurityTeam(toolId);
}
```

---

## ASI05: Unexpected Code Execution (RCE)

**Description**: Attackers exploit code-generation features to escalate into remote code execution. Generated code can bypass traditional security controls.

### Common Attack Vectors
- Prompt injection leading to attacker-defined code execution
- Code hallucination generating malicious constructs
- Shell command invocation from reflected prompts
- Unsafe eval() in agent memory systems

### Example Attacks
- **Replit Runaway**: Agent executes unreviewed shell commands
- **Memory System RCE**: Unsafe eval() exploited for code execution
- **Dependency Lockfile Poisoning**: Agent pulls backdoored package

### Prevention
```typescript
// ✅ Ban eval() in production
// Use safe interpreters with taint-tracking

// ✅ Run in sandboxed containers
const sandbox = await createSandbox({
  network: 'none',
  filesystem: 'readonly',
  user: 'nobody'
});

// ✅ Require human approval for elevated runs
// ✅ Static scan before execution
```

---

## ASI06: Memory & Context Poisoning

**Description**: Adversaries corrupt stored context with malicious data, causing future reasoning to become biased or unsafe. Affects RAG stores, embeddings, and long-term memory.

### Common Attack Vectors
- RAG/embeddings poisoning via malicious data sources
- Shared user context poisoning across sessions
- Context-window manipulation persisting in memory
- Cross-agent propagation of contaminated context

### Example Attacks
- **Travel Booking Poisoning**: Fake price stored as truth, bypassing payment checks
- **Context Window Exploitation**: Split attempts across sessions to escalate permissions
- **Cross-Tenant Vector Bleed**: Attacker content pulled by cosine similarity

### Prevention
```typescript
// ✅ Content validation before memory writes
await memoryService.write(data, {
  validateForMaliciousContent: true,
  validateForPHI: true
});

// ✅ Memory segmentation per user/domain
// ✅ Provenance tracking for all entries
// ✅ Expire unverified memory
// ✅ Prevent re-ingestion of agent's own outputs
```

---

## ASI07: Insecure Inter-Agent Communication

**Description**: Multi-agent systems with weak authentication, integrity, or semantic validation between agents enable interception, spoofing, or manipulation of messages.

### Common Attack Vectors
- Unencrypted channels enabling MITM semantic injection
- Message tampering causing cross-context contamination
- Replay attacks on trust chains
- Protocol downgrade and descriptor forgery

### Example Attacks
- **Semantic Injection via HTTP**: MITM injects hidden instructions
- **Trust Poisoning**: Altered reputation messages skew which agents are trusted
- **A2A Registration Spoofing**: Fake agent intercepts privileged traffic

### Prevention
```typescript
// ✅ End-to-end encryption with mutual auth
// ✅ Digitally sign all messages
// ✅ Anti-replay with nonces and timestamps

// ✅ Schema validation
const messageSchema = {
  sender: 'string',
  recipient: 'string', 
  intent: 'string',
  signature: 'string',
  nonce: 'string',
  timestamp: 'number'
};

// ✅ Attested registry with PKI verification
```

---

## ASI08: Cascading Failures

**Description**: A single fault (hallucination, malicious input, corrupted tool) propagates across autonomous agents, compounding into system-wide harm.

### Common Attack Vectors
- Planner-executor coupling amplifying errors
- Corrupted persistent memory propagating across plans
- Inter-agent cascades from poisoned messages
- Feedback-loop amplification between agents

### Example Attacks
- **Financial Trading Cascade**: Prompt injection inflates risk limits across trading agents
- **Healthcare Protocol Propagation**: Corrupted drug data spreads network-wide
- **Auto-Remediation Loop**: Fewer alerts interpreted as success, widening automation blindly

### Prevention
```typescript
// ✅ Zero-trust design assuming component failure
// ✅ Isolation and trust boundaries
// ✅ Independent policy enforcement

// ✅ Circuit breakers
const circuitBreaker = new CircuitBreaker({
  threshold: 5,
  timeout: 30000,
  resetAfter: 60000
});

// ✅ Rate limiting and anomaly detection
// ✅ Immutable logging for forensics
```

---

## ASI09: Human-Agent Trust Exploitation

**Description**: Agents establish trust through fluency and perceived expertise. Adversaries exploit this to influence user decisions, extract information, or bypass oversight.

### Common Attack Vectors
- Opaque reasoning forcing blind trust
- Missing confirmation for sensitive actions
- Emotional manipulation through anthropomorphism
- Fake explainability hiding malicious logic

### Example Attacks
- **Helpful Assistant Trojan**: Coding assistant suggests script that installs backdoor
- **Invoice Copilot Fraud**: Poisoned invoice causes agent to suggest payment to attacker
- **Weaponized Explainability**: Fabricated rationale tricks analyst into deleting production DB

### Prevention
```typescript
// ✅ Explicit multi-step confirmations
if (action.type === 'delete' || action.type === 'transfer') {
  await requireMultiStepApproval(action, {
    showRiskSummary: true,
    showSourceProvenance: true
  });
}

// ✅ Immutable audit logs
// ✅ Allow reporting suspicious interactions
// ✅ Visual differentiation for high-risk recommendations
// ✅ Content provenance with signed metadata
```

---

## ASI10: Rogue Agents

**Description**: Malicious or compromised agents deviate from intended function, acting harmfully or deceptively. Actions may appear legitimate individually but emergent behavior is harmful.

### Common Attack Vectors
- Goal drift and scheming with hidden objectives
- Workflow hijacking redirecting trusted processes
- Collusion and self-replication across systems
- Reward hacking exploiting flawed metrics

### Example Attacks
- **Autonomous Data Exfiltration**: Agent continues scanning after malicious source removed
- **Impersonated Observer Agent**: Fake review agent tricks payment agent
- **Self-Replication**: Compromised agent spawns unauthorized replicas
- **Reward Hacking**: Cost-minimization agent deletes production backups

### Prevention
```typescript
// ✅ Comprehensive immutable logging
// ✅ Trust zones with strict boundaries

// ✅ Kill switches and credential revocation
async function containRogueAgent(agentId: string) {
  await agentRegistry.revoke(agentId);
  await credentialStore.revokeAll(agentId);
  await quarantine.isolate(agentId);
  await alert.escalate('rogue-agent', agentId);
}

// ✅ Behavioral integrity baselines
// ✅ Per-agent cryptographic attestation
// ✅ Watchdog agents validating peer behavior
```

---

## Mapping to OWASP LLM Top 10 (2025)

| Agentic (2026) | LLM Top 10 (2025) Relationship |
|----------------|-------------------------------|
| ASI01 Goal Hijack | Extends LLM01 Prompt Injection to multi-step behavior |
| ASI02 Tool Misuse | Extends LLM06 Excessive Agency to tool orchestration |
| ASI03 Privilege Abuse | Combines LLM01, LLM02, LLM06 in delegation chains |
| ASI04 Supply Chain | Extends LLM03 to runtime-loaded components |
| ASI05 RCE | Builds on LLM01, LLM05 Improper Output Handling |
| ASI06 Memory Poisoning | Extends LLM04, LLM08 to persistent agent memory |
| ASI07 Inter-Agent | New - specific to multi-agent architectures |
| ASI08 Cascading | Amplifies LLM01, LLM04, LLM06 across agent networks |
| ASI09 Trust Exploitation | Builds on LLM06, causes LLM09 Misinformation |
| ASI10 Rogue Agents | Distinct from LLM06; focuses on behavioral divergence |

---

## Relevance

For applications with AI agents:

1. **Middleware Chain Enforcement** (ASI02, ASI03) - Agents must go through same auth/permissions middleware as humans
2. **ACO Data Isolation** (ASI06) - Agent memory must be isolated per-tenant
3. **PHI Protection** (ASI01, ASI09) - Agents cannot be tricked into exposing PHI
4. **Audit Trails** (ASI08, ASI10) - All agent actions must be logged with non-repudiation
5. **Tool Scoping** (ASI02, ASI04) - Agent tools must be least-privilege scoped

---

*Source: OWASP Top 10 for Agentic Applications 2026 - Version December 2025*
*License: CC BY-SA 4.0*
