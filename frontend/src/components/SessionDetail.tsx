import React from 'react';
import { Box, Text } from 'ink';
import useStore from '../store.js';
import { parseMarkdownSections } from '../utils/markdown.js';

export default function SessionDetail() {
  const session = useStore((s) => s.currentSession);
  if (!session) return <Text>No session selected.</Text>;

  const { command, stdout, stderr, exit_code, created_at, ai_response } = session as any;
  // ai_response: use parseMarkdownSections
  let parsed: { header: string, content: string }[] = [];
  if (ai_response) parsed = parseMarkdownSections(ai_response);

  return (
    <Box flexDirection="column">
      <Text>Session: {command}  <Text dimColor>{created_at?.slice(0,19).replace('T',' ')}</Text> <Text dimColor>[{exit_code}]</Text></Text>
      <Box flexDirection="column" marginTop={1}>
        {stdout && (
          <Text color="green">stdout: {stdout.slice(0, 512) + (stdout.length > 512 ? '...' : '')}</Text>
        )}
        {stderr && (
          <Text color="red">stderr: {stderr.slice(0, 512) + (stderr.length > 512 ? '...' : '')}</Text>
        )}
      </Box>
      <Box flexDirection="column" marginTop={1}>
        <Text>AI Solution:</Text>
        {parsed.length > 0
          ? parsed.map((sect, i) => (
              <Box key={i} flexDirection="column" marginBottom={1}>
                <Text underline>{sect.header}</Text>
                <Text>{sect.content}</Text>
              </Box>
            ))
          : ai_response && <Text>{ai_response}</Text>}
      </Box>
    </Box>
  );
}
