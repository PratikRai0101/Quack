import React from 'react';
import { Box, Text } from 'ink';
import useStore from '../store';

export default function ErrorOutput() {
  const stdout = useStore((s) => s.stdout);
  const stderr = useStore((s) => s.stderr);
  const command = useStore((s) => s.command);

  return (
    <Box flexDirection="column" padding={1} borderStyle="round" borderColor="gray">
      <Text bold>Analyzing: {command ?? '<no command>'}</Text>
      {stderr ? (
        <Box marginTop={1} flexDirection="column">
          <Text color="red">stderr:</Text>
          <Text>{stderr}</Text>
        </Box>
      ) : null}

      {stdout ? (
        <Box marginTop={1} flexDirection="column">
          <Text color="green">stdout:</Text>
          <Text>{stdout}</Text>
        </Box>
      ) : null}
    </Box>
  );
}
