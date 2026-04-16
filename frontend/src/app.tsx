import React from 'react';
import { Box, Text } from 'ink';
import Header from './components/Header.js';
import ErrorOutput from './components/ErrorOutput.js';
import useStore from './store.js';

export default function App({ command }: { command?: string }) {
  const [status, setStatus] = React.useState<string>('checking');
  const analyze = useStore((s) => s.analyze);
  const sessionId = useStore((s) => s.sessionId);

  React.useEffect(() => {
    fetch('http://localhost:3001/api/health')
      .then((res) => res.json())
      .then(() => setStatus('connected'))
      .catch(() => setStatus('no-backend'));
  }, []);

  React.useEffect(() => {
    if (command && !sessionId) {
      // auto-run analyze when command provided on startup
      analyze(command).catch(() => {});
    }
  }, [command, sessionId]);

  return (
    <Box flexDirection="column">
      <Header />
      <Box marginTop={1} />
      <ErrorOutput />
      <Box marginTop={1} />
      <Text dimColor>Type y to copy solution, r to re-analyze, q to quit</Text>
    </Box>
  );
}
