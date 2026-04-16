import React from 'react';
import { Box, Text } from 'ink';

export default function App({ command }: { command?: string }) {
  const [status, setStatus] = React.useState<string>('checking');

  React.useEffect(() => {
    fetch('http://localhost:3001/api/health')
      .then((res) => res.json())
      .then(() => setStatus('connected'))
      .catch(() => setStatus('no-backend'));
  }, []);

  return (
    <Box flexDirection="column">
      <Text>🦆 Quack v2.0 - status: {status}</Text>
      <Text>Command: {command ?? 'none'}</Text>
    </Box>
  );
}
