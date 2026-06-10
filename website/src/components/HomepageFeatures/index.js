import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

const FeatureList = [
  {
    title: 'Full Session Lifecycle',
    description: (
      <>
        Create, resume, list, and delete sessions. Switch models and modes
        mid-conversation, manage plans, agents, and foreground control through a
        typed async API built on Tokio.
      </>
    ),
  },
  {
    title: 'Streaming Events',
    description: (
      <>
        Subscribe to 40+ strongly-typed streaming events: assistant messages and
        deltas, reasoning, tool execution, usage, compaction, and session
        lifecycle - all parsed into Rust enums.
      </>
    ),
  },
  {
    title: 'Custom Tools and Permissions',
    description: (
      <>
        Register tools with a fluent builder, handle invocations with async
        handlers, and control approvals with permission callbacks or
        allow/deny lists.
      </>
    ),
  },
  {
    title: 'Hooks and User Input',
    description: (
      <>
        Intercept the session lifecycle with six hooks (pre/post tool use, prompt
        submit, session start/end, error) and answer interactive user-input
        requests from the agent.
      </>
    ),
  },
  {
    title: 'BYOK and MCP Servers',
    description: (
      <>
        Bring your own API keys for OpenAI-compatible providers, supply a custom
        model list, and connect local or remote Model Context Protocol servers.
      </>
    ),
  },
  {
    title: 'Telemetry and Infinite Sessions',
    description: (
      <>
        Configure OpenTelemetry export for the CLI process and enable infinite
        sessions with automatic and manual context compaction.
      </>
    ),
  },
];

function Feature({title, description}) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
