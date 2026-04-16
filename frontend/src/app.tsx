import React from 'react';
import { Box, Text } from 'ink';
import Header from './components/Header.js';
import ErrorOutput from './components/ErrorOutput.js';
import StreamingText from './components/StreamingText.js';
import InputPrompt from './components/InputPrompt.js';
import useStore from './store.js';

import { useInput } from 'ink';

export default function App({ command }: { command?: string }) {
  const [status, setStatus] = React.useState<string>('checking');
  const analyze = useStore((s) => s.analyze);
  const sessionId = useStore((s) => s.sessionId);

  const inputActive = useStore((s) => s.inputActive);
  useInput((input, key) => {
    if (inputActive) return; // user typing in InputPrompt
    if (input === 'q' || key.escape) process.exit(0);
    if (input === 'y') useStore.getState().copyFix();
    if (input === 'r') useStore.getState().reAnalyze();
    if (key.upArrow) {/* future: scroll up */}
    if (key.downArrow) {/* future: scroll down */}
  });

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
      <StreamingText />
      <Box marginTop={1} />
      <InputPrompt />
      <Box marginTop={1} />
      <Text dimColor>Type y to copy solution, r to re-analyze, q to quit</Text>
    </Box>
  );
}
