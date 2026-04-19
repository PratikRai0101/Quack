import React from 'react';
import { Box, Text } from 'ink';
import useStore from '../store.js';
import { parseMarkdownSections } from '../utils/markdown.js';
import SolutionBlock from './SolutionBlock.js';

export default function StreamingText() {
  const ai = useStore(s => s.aiResponse);
  const isStreaming = useStore(s => s.isStreaming);
  const sections = React.useMemo(() => parseMarkdownSections(ai), [ai]);

  return (
    <Box flexDirection="column" padding={1} borderStyle="round" borderColor="gray">
      {sections.length === 0 ? (
        <Text dimColor>{isStreaming ? 'Waiting for analysis...' : 'No analysis yet'}</Text>
      ) : (
        sections.map((sec, i) => {
          if (sec.type === 'solution') return <SolutionBlock key={i} content={sec.content} />;
          const color = sec.type === 'glitch' ? 'red' : sec.type === 'protip' ? 'cyan' : undefined;
          return <Box key={i} marginBottom={1}><Text color={color}>{sec.content}</Text></Box>;
        })
      )}
      {isStreaming && <Text dimColor>● streaming…</Text>}
    </Box>
  );
}
