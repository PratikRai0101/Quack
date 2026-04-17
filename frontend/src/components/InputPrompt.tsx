import React, { useState } from 'react';
import { Box, Text } from 'ink';
import TextInput from 'ink-text-input';
import useStore from '../store.js';

export default function InputPrompt() {
  const [value, setValue] = useState('');
  const followUp = useStore((s) => s.followUp);
  const appendChunk = useStore((s) => s.appendChunk);
  const setInputActive = useStore((s) => s.setInputActive);
  const backendUrl = useStore((s) => s.backendUrl);

  React.useEffect(() => () => setInputActive(false), [setInputActive]);

  const handleSetCommand = async (key: string, val: string) => {
    // only allow certain keys
    const allowed = ['api_key', 'provider', 'model', 'base_url'];
    if (!allowed.includes(key)) {
      appendChunk(`\n\n[Unknown setting: ${key}]`);
      return;
    }
    try {
      const payload: any = {};
      payload[key] = val;
      const res = await fetch(`${backendUrl}/api/settings`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      if (!res.ok) {
        appendChunk(`\n\n[Settings update failed: ${res.status} ${res.statusText}]`);
        return;
      }
      const updated = await res.json();
      appendChunk(`\n\n[Settings updated]`);
    } catch (e: any) {
      appendChunk(`\n\n[Settings update error: ${e.message ?? e}]`);
    }
  };

  const onSubmit = async (input: string) => {
    setInputActive(false);
    const q = input.trim();
    if (!q) return;

    // echo user question into the log
    appendChunk(`\n\n> ${q}\n`);

    // Command parsing: set / history / view
    const parts = q.split(/\s+/);
    const cmd = parts[0].toLowerCase();

    if (cmd === 'set' && parts.length >= 3) {
      const key = parts[1];
      const val = parts.slice(2).join(' ');
      await handleSetCommand(key, val);
      setValue('');
      return;
    }

    if (cmd === 'history') {
      try {
        await useStore.getState().loadHistory();
        appendChunk('\n\n[History loaded]');
      } catch (e: any) {
        appendChunk(`\n\n[Failed to load history: ${e.message ?? e}]`);
      }
      setValue('');
      return;
    }

    if (cmd === 'view' && parts.length >= 2) {
      const id = parts[1];
      try {
        await useStore.getState().loadSession(id);
        appendChunk(`\n\n[Loaded session ${id}]`);
      } catch (e: any) {
        appendChunk(`\n\n[Failed to load session: ${e.message ?? e}]`);
      }
      setValue('');
      return;
    }

    // Fallback: regular follow-up
    if (followUp) {
      try {
        await followUp(q);
      } catch (e) {
        appendChunk('\n\n[Follow-up failed to start]\n');
      }
    } else {
      appendChunk('\n\n[Follow-up stubbed: backend not available]\n');
    }

    setValue('');
  };

  return (
    <Box>
      <Text dimColor>&gt; </Text>
      <TextInput value={value} onChange={(val) => { setValue(val); setInputActive(val.length > 0); }} onSubmit={onSubmit} />
    </Box>
  );
}
