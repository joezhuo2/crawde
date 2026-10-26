import { mkdirSync, writeFileSync } from 'node:fs';
import { deflateSync } from 'node:zlib';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const pixels = [
  '................',
  '................',
  '.OO..........OO.',
  '.OOO........OOO.',
  '.O.O..O..O..O.O.',
  '..OO..W..W..OO..',
  '...OOOOOOOOOO...',
  '....OOOOOOOO....',
  '...OOOOOOOOOO...',
  '..OO.OOOOOO.OO..',
  '.OO..OOOOOO..OO.',
  '.O...O....O...O.',
  '.....O....O.....',
  '................',
  '................',
  '................',
];
const palette = { O: [241, 139, 96, 255], W: [255, 246, 232, 255], '.': [0, 0, 0, 0] };
if (pixels.length !== 16 || pixels.some(row => row.length !== 16)) throw new Error('Brand mark must fit the 16px grid');

function crc32(buffer) {
  let crc = 0xffffffff;
  for (const byte of buffer) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const name = Buffer.from(type);
  const length = Buffer.alloc(4); length.writeUInt32BE(data.length);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(Buffer.concat([name, data])));
  return Buffer.concat([length, name, data, crc]);
}

function png(size) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0); header.writeUInt32BE(size, 4); header[8] = 8; header[9] = 6;
  const raw = Buffer.alloc(size * (size * 4 + 1));
  for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) {
    const color = palette[pixels[Math.floor(y * 16 / size)][Math.floor(x * 16 / size)]];
    raw.set(color, y * (size * 4 + 1) + 1 + x * 4);
  }
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', header), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]);
}

const rects = pixels.flatMap((row, y) => [...row].flatMap((pixel, x) => pixel === '.' ? [] : [
  `<rect x="${x}" y="${y}" width="1" height="1" fill="${pixel === 'W' ? '#fff6e8' : '#f18b60'}"/>`,
])).join('');
const mark = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" shape-rendering="crispEdges" role="img" aria-label="Crawde crab">${rects}</svg>\n`;
const banner = `<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="320" viewBox="0 0 1280 320" role="img" aria-labelledby="title description">
  <title id="title">Crawde</title>
  <desc id="description">A little company while Claude works.</desc>
  <rect width="1280" height="320" rx="16" fill="#121416"/>
  <g transform="translate(232 72) scale(10)" shape-rendering="crispEdges">${rects}</g>
  <text x="440" y="175" fill="#f6f3ed" font-family="Segoe UI, Arial, sans-serif" font-weight="700" font-size="76" letter-spacing="-3">Crawde</text>
  <text x="445" y="220" fill="#aaa7a1" font-family="Segoe UI, Arial, sans-serif" font-size="22">A little company while Claude works.</text>
</svg>\n`;

mkdirSync(resolve(root, 'assets'), { recursive: true });
mkdirSync(resolve(root, 'src-tauri/icons'), { recursive: true });
writeFileSync(resolve(root, 'assets/crawde-mark.svg'), mark);
writeFileSync(resolve(root, 'assets/crawde-banner.svg'), banner);
for (const size of [16, 24, 32]) writeFileSync(resolve(root, `src-tauri/icons/tray${size === 32 ? '' : `-${size}`}.png`), png(size));
console.log('Generated Crawde banner, brand mark, and 16/24/32px tray icons.');
