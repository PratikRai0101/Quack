import React from 'react';
import { Box, Text } from 'ink';
import useStore from '../store.js';

const MASK = (k: string | undefined) => (k ? '••••••••' + k.slice(-4) : '');

export default function SettingsView() {
  const backendUrl = useStore((s) => s.backendUrl);
  const [settings, setSettings] = React.useState<any>(null);
  const [loading, setLoading] = React.useState<boolean>(true);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    let cancelled = false;
    setLoading(true); setSettings(null); setError(null);
    fetch(`${backendUrl}/api/settings`)
      .then(r => r.json())
      .then(s => { if (!cancelled) { setSettings(s); setLoading(false); }})
      .catch(e => { if (!cancelled) { setError(e.message ?? String(e)); setLoading(false); }});
    return () => { cancelled = true; };
  }, [backendUrl]);

  if (loading) return <Text>Loading settings...</Text>;
  if (error) return <Text color="red">Settings load error: {error}</Text>;

  return (
    <Box flexDirection="column">
      <Text>Quack Settings:</Text>
      <Box>
        <Text dimColor>api_key:</Text> <Text>{MASK(settings.api_key)}</Text>
      </Box>
      <Box>
        <Text dimColor>provider:</Text> <Text>{settings.provider || ''}</Text>
      </Box>
      <Box>
        <Text dimColor>model:</Text> <Text>{settings.model || ''}</Text>
      </Box>
      <Box>
        <Text dimColor>base_url:</Text> <Text>{settings.base_url || ''}</Text>
      </Box>
      <Box marginTop={1} flexDirection="column">
        <Text dimColor>To change: use "set api_key <key>", "set provider <prov>", "set model <name>", "set base_url <url>"</Text>
      </Box>
    </Box>
  );
}
