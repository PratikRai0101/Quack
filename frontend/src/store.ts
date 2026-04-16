import create from 'zustand';
import EventSource from 'eventsource';
import clipboardy from 'clipboardy';
import { extractFirstFencedCode } from './utils/markdown';

type State = {
  backendUrl: string;
  sessionId?: string | null;
  command?: string | null;
  stdout: string;
  stderr: string;
  aiResponse: string;
  isAnalyzing: boolean;
  isStreaming: boolean;
  es: any | null;
  analyze: (command: string) => Promise<void>;
  appendChunk: (chunk: string) => void;
  copyFix: () => Promise<boolean>;
  reAnalyze: () => Promise<void>;
  reset: () => void;
};

export const useStore = create<State>((set, get) => ({
  backendUrl: process.env.QUACK_BACKEND_URL ?? 'http://localhost:3001',
  sessionId: null,
  command: null,
  stdout: '',
  stderr: '',
  aiResponse: '',
  isAnalyzing: false,
  isStreaming: false,
  es: null,

  analyze: async (command: string) => {
    const base = get().backendUrl;
    set({ isAnalyzing: true, isStreaming: true, aiResponse: '', command });
    try {
      const res = await fetch(`${base}/api/analyze`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command })
      });
      if (!res.ok) throw new Error(`Analyze failed: ${res.statusText}`);
      const data = await res.json();
      set({ sessionId: data.session_id, stdout: data.stdout ?? '', stderr: data.stderr ?? '' });

      const url = `${base}/api/analyze/${data.session_id}/stream`;
      const es = new EventSource(url);
      set({ es });

      es.addEventListener('chunk', (evt: any) => {
        try {
          const payload = JSON.parse(evt.data);
          if (payload && payload.content) get().appendChunk(payload.content);
          else get().appendChunk(evt.data);
        } catch {
          get().appendChunk(evt.data);
        }
      });

      es.addEventListener('done', () => {
        try { es.close(); } catch {}
        set({ isStreaming: false, isAnalyzing: false, es: null });
      });

      es.onerror = (err: any) => {
        console.error('SSE error', err);
        try { es.close(); } catch {}
        set({ isStreaming: false, isAnalyzing: false, es: null });
        get().appendChunk('\n\n[Stream error: connection dropped]');
      };
    } catch (err: any) {
      console.error('analyze error', err);
      set({ isStreaming: false, isAnalyzing: false });
      get().appendChunk(`\n\n[Error starting analysis: ${err.message ?? err}]`);
    }
  },

  appendChunk: (chunk: string) => set((s) => ({ aiResponse: s.aiResponse + chunk })),

  copyFix: async () => {
    const text = get().aiResponse;
    const sol = extractFirstFencedCode(text);
    if (!sol) return false;
    await clipboardy.write(sol.code);
    return true;
  },

  reAnalyze: async () => {
    const cmd = get().command;
    const es = get().es;
    if (es) try { es.close(); set({ es: null }); } catch {}
    if (cmd) await get().analyze(cmd);
  },

  reset: () => set({ sessionId: null, command: null, stdout: '', stderr: '', aiResponse: '', isAnalyzing: false, isStreaming: false })
}));

export default useStore;
