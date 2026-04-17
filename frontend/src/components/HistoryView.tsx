import React from 'react';
import { Box, Text, useInput } from 'ink';
import useStore from '../store.js';

export default function HistoryView() {
  const history = useStore((s) => s.history);
  const historyLoading = useStore((s) => s.historyLoading);
  const historyError = useStore((s) => s.historyError);
  const loadSession = useStore((s) => s.loadSession);
  const setUiMode = useStore((s) => s.setUiMode);

  const [selected, setSelected] = React.useState(0);

  useInput((input, key) => {
    if (key.upArrow) setSelected((i) => Math.max(0, i - 1));
    else if (key.downArrow) setSelected((i) => Math.min((history?.length ?? 1) - 1, i + 1));
    else if (key.return && history && history[selected]) {
      loadSession(history[selected].id);
      setUiMode('session');
    }
  });

  if (historyLoading) return <Text>Loading history...</Text>;
  if (historyError) return <Text color="red">History error: {historyError}</Text>;
  if (!history || history.length === 0) return <Text>No previous sessions found.</Text>;

  return (
    <Box flexDirection="column">
      <Text>Session History:</Text>
      {history.map((s, i) => (
        <Box key={s.id}>
          <Text color={i === selected ? 'cyan' : undefined}>
            {i === selected ? '> ' : '  '}
            {s.command ?? ''} 
            <Text dimColor>({s.created_at?.slice(0,19).replace('T',' ') ?? ''})</Text>
            <Text dimColor> [{s.exit_code!==undefined?s.exit_code:'?'}]</Text>
          </Text>
        </Box>
      ))}
      <Box marginTop={1}><Text dimColor>↑/↓ to select, Enter to view</Text></Box>
    </Box>
  );
}
