import React from 'react';
import { Box, Text } from 'ink';
import useStore from '../store';

export default function Header() {
  const isStreaming = useStore((s) => s.isStreaming);
  const command = useStore((s) => s.command);

  return (
    <Box justifyContent="space-between">
      <Text color="cyan">🦆 Quack v2.0</Text>
      <Box>
        <Text> {command ? `Command: ${command}` : ''} </Text>
        <Text> {isStreaming ? ' ● streaming' : ''}</Text>
      </Box>
    </Box>
  );
}
