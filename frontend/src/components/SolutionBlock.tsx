import React from 'react';
import { Box, Text } from 'ink';
import { extractFirstFencedCode } from '../utils/markdown.js';
import useStore from '../store.js';

export default function SolutionBlock({ content }: { content: string }) {
  const snippet = React.useMemo(() => extractFirstFencedCode(content), [content]);

  return (
    <Box flexDirection="column" borderStyle="single" borderColor="green" padding={1}>
      <Box justifyContent="space-between">
        <Text color="green" bold>Solution</Text>
        <Text dimColor> [y] copy</Text>
      </Box>

      {snippet ? (
        <Box marginTop={1} flexDirection="column"><Text>{snippet.code}</Text></Box>
      ) : (
        <Box marginTop={1}><Text dimColor>No fenced code found.</Text></Box>
      )}

      <Box marginTop={1}><Text dimColor>Press 'y' to copy the solution to clipboard</Text></Box>
    </Box>
  );
}
