import React from 'react';
import { Box, Text } from 'ink';

export default function CodeBlock({ code, language }: { code: string; language?: string }) {
  // Minimal code block renderer. Preserves whitespace by splitting lines.
  return (
    <Box flexDirection="column" borderStyle="single" borderColor="white" padding={1}>
      {code.split('\n').map((line, i) => (
        <Text key={i}>{line}</Text>
      ))}
    </Box>
  );
}
