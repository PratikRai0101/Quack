import create from 'zustand';
import EventSource from 'eventsource';
import clipboardy from 'clipboardy';
import { extractFirstFencedCode } from './utils/markdown';

type HistorySummary = {
  id: string;
  command?: string;
  exit_code?: number;
  created_at?: string;
};

// Likely shape; update to openapi type on next regenerate
type HistorySessionDetail = {
  id: string;
  command?: string;
  stdout?: string;
  stderr?: string;
  exit_code?: number;
  created_at?: string;
};

type UiMode = 'default' | 'settings' | 'history' | 'session';

type State = {
  // History/session state
  history: null | HistorySummary[];
  currentSession: null | HistorySessionDetail;
  historyLoading: boolean;
  sessionLoading: boolean;
   historyError?: string | null;
   sessionError?: string | null;

   uiMode: UiMode;
   setUiMode: (m: UiMode) => void;
  backendUrl: string;
  sessionId?: string | null;
  command?: string | null;
  stdout: string;
  stderr: string;
  aiResponse: string;
  isAnalyzing: boolean;
  isStreaming: boolean;
  es: any | null;
  inputActive: boolean;
  setInputActive: (b: boolean) => void;
  analyze: (command: string) => Promise<void>;
  appendChunk: (chunk: string) => void;
  copyFix: () => Promise<boolean>;
  reAnalyze: () => Promise<void>;
  reset: () => void;
  followUp: (question: string) => Promise<void>;
};

export const useStore = create<State>((set, get) => ({
  backendUrl: process.env.QUACK_BACKEND_URL ?? 'http://localhost:3001',

  uiMode: 'default',
  setUiMode: (m: UiMode) => set({ uiMode: m }),

  // --- History/session state ---
  history: null,
  currentSession: null,
  historyLoading: false,
  sessionLoading: false,
  historyError: null,
  sessionError: null,
  sessionId: null,
  command: null,
  stdout: '',
  stderr: '',
  aiResponse: '',
  isAnalyzing: false,
  isStreaming: false,
  es: null,
  inputActive: false,
   setInputActive: (b: boolean) => set({ inputActive: b }),

   loadHistory: async () => {
     set({ historyLoading: true, historyError: null });
     const base = get().backendUrl;
     try {
       const res = await fetch(`${base}/api/history`);
       if (!res.ok) throw new Error(`Failed to load history: ${res.statusText}`);
       const data = await res.json();
       // Defensive, ensure array
       set({ history: Array.isArray(data) ? data : data && data.length ? data : [], historyLoading: false });
     } catch (err: any) {
       set({ historyError: err.message ?? String(err), historyLoading: false });
     }
   },

   loadSession: async (id: string) => {
     set({ sessionLoading: true, sessionError: null });
     const base = get().backendUrl;
     try {
       const res = await fetch(`${base}/api/history/${id}`);
       if (!res.ok) throw new Error(`Failed to load session: ${res.statusText}`);
       const data = await res.json();
       set({ currentSession: data, sessionLoading: false });
     } catch (err: any) {
       set({ sessionError: err.message ?? String(err), sessionLoading: false });
     }
   },


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

    reset: () => set({
     sessionId: null, command: null, stdout: '', stderr: '', aiResponse: '', isAnalyzing: false, isStreaming: false,
     history: null, currentSession: null, historyLoading: false, sessionLoading: false, historyError: null, sessionError: null
   }),

    followUp: async (question: string) => {
      const base = get().backendUrl;
      const sessionId = get().sessionId;
      if (!sessionId) {
        get().appendChunk("\n\n### Follow-up (stubbed)\nNo active session. Your question: > " + question + "\n");
        return;
      }
      // Close any open EventSource
      const prevEs = get().es;
      if (prevEs) try { prevEs.close(); set({ es: null }); } catch {}
      set({ isStreaming: true });
      let url = '';
      try {
        const res = await fetch(`${base}/api/followup`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ session_id: sessionId, question })
        });
        if (!res.ok) throw new Error(`Follow-up failed: ${res.statusText}`);
        const data = await res.json();
        url = data.stream_url;
        if (url && url.startsWith('/')) url = base + url;
        if (!url) throw new Error('No stream_url in response');
      } catch (e: any) {
        get().appendChunk("\n\n### Follow-up (stubbed)\nThe backend does not provide /api/followup in dev mode. This is a simulated reply for frontend testing.\n");
        set({ isStreaming: false });
        return;
      }
      // Open stream if url
      try {
        const es = new EventSource(url);
        set({ es });
        es.addEventListener('chunk', (evt: any) => {
          try {
            const payload = JSON.parse(evt.data);
            if (payload && payload.content) get().appendChunk(payload.content);
            else get().appendChunk(evt.data);
          } catch { get().appendChunk(evt.data); }
        });
        es.addEventListener('done', () => { try { es.close(); } catch {} set({ isStreaming: false, es: null }); });
        es.onerror = (err: any) => {
          console.error('SSE error', err);
          try { es.close(); } catch {}
          set({ isStreaming: false, es: null });
          get().appendChunk('\n\n[Stream error: connection dropped]');
        };
      } catch (e: any) {
        get().appendChunk("\n\n[Error establishing stream: " + (e.message ?? e) + "]\n");
        set({ isStreaming: false });
      }
    }
}));

export default useStore;
