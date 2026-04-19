import { extractFirstFencedCode, parseMarkdownSections } from './markdown.js';

function assert(cond: boolean, msg: string) {
  if (!cond) {
    console.error('FAIL:', msg);
    process.exit(1);
  }
}

// Test extractFirstFencedCode
const md1 = '### The Solution\n```rust\nlet x: i32 = 42;\n```\n';
const res1 = extractFirstFencedCode(md1);
assert(res1 !== null, 'extractFirstFencedCode should return a snippet for fenced code');
assert(res1!.code.includes('let x: i32 = 42;'), 'snippet.code should contain the code');

// Test parseMarkdownSections
const md2 = '### Analysis\nWe tried to run the command.\n### The Glitch\nA type error occurred.\n### The Solution\n```js\nconsole.log("hi")\n```\n### Pro-Tip\nUse cargo check.';
const secs = parseMarkdownSections(md2);
assert(secs.length >= 4, `expected at least 4 sections, got ${secs.length}`);
assert(secs.some(s => s.type === 'analysis'), 'should contain analysis section');
assert(secs.some(s => s.type === 'glitch'), 'should contain glitch section');
assert(secs.some(s => s.type === 'solution'), 'should contain solution section');
assert(secs.some(s => s.type === 'protip'), 'should contain protip section');

console.log('All markdown utils tests passed.');
process.exit(0);
