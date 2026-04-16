import fetch from 'node-fetch';
import EventSource from 'eventsource';

async function main() {
  const base = process.env.QUACK_BACKEND_URL || 'http://localhost:3001';
  console.log('Using backend:', base);

  const res = await fetch(`${base}/api/analyze`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ command: 'ls /nonexistent' })
  });

  if (!res.ok) {
    console.error('Analyze POST failed with status', res.status);
    process.exit(2);
  }

  const data = await res.json();
  const sessionId = data.session_id;
  if (!sessionId) {
    console.error('No session_id in response');
    process.exit(2);
  }
  console.log('Got session id:', sessionId);

  const url = `${base}/api/analyze/${sessionId}/stream`;
  console.log('Connecting to SSE:', url);
  const es = new EventSource(url);

  const timeout = setTimeout(() => {
    console.error('Timeout waiting for SSE chunk');
    es.close();
    process.exit(2);
  }, 10000);

  es.addEventListener('chunk', (evt) => {
    console.log('SSE chunk received:', evt.data);
    clearTimeout(timeout);
    es.close();
    process.exit(0);
  });

  es.addEventListener('error', (err) => {
    console.error('SSE error:', err);
    clearTimeout(timeout);
    es.close();
    process.exit(2);
  });
}

main().catch(err => { console.error(err); process.exit(2); });
