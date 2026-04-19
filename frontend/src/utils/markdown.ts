export function extractFirstFencedCode(md: string): { language?: string; code: string } | null {
  if (!md) return null;
  const m = md.match(/```(\w+)?\n([\s\S]*?)\n```/);
  if (!m) return null;
  return { language: m[1] || 'text', code: m[2] };
}

export type Section = { type: 'analysis'|'glitch'|'solution'|'protip'|'text'; content: string };

export function parseMarkdownSections(md: string): Section[] {
  if (!md) return [];
  // Split on headers that start with ### (keep header with block)
  const parts = md.split(/(?=^###\s)/m);
  return parts.map((p) => {
    const headerMatch = p.match(/^###\s*\**\s*([^\n*]+)\**\s*\n?/i);
    if (!headerMatch) return { type: 'text', content: p };
    const title = headerMatch[1].toLowerCase();
    const body = p.slice(headerMatch[0].length).trim();
    if (title.includes('glitch')) return { type: 'glitch', content: body };
    if (title.includes('solution')) return { type: 'solution', content: body };
    if (title.includes('analysis')) return { type: 'analysis', content: body };
    if (title.includes('pro-tip') || title.includes('pro tip')) return { type: 'protip', content: body };
    return { type: 'text', content: body };
  });
}
