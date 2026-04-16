import React, { useState } from 'react';
import { Box, Text } from 'ink';
import TextInput from 'ink-text-input';
import useStore from '../store.js';

export default function InputPrompt() {
  const [value, setValue] = useState('');
  const followUp = useStore((s) => s.followUp);
  const appendChunk = useStore((s) => s.appendChunk);
  const setInputActive = useStore((s) => s.setInputActive);
  React.useEffect(() => () => setInputActive(false), [setInputActive]);

  const onSubmit = async (input: string) => {
    setInputActive(false);
    const q = input.trim();
    if (!q) return;
    // echo user question into the log
    appendChunk(`\n\n> ${q}\n`);
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
