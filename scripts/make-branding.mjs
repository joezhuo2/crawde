import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Resvg } from '@resvg/resvg-js';

const root = fileURLToPath(new URL('../', import.meta.url));
// Palette sampled from Crab_Idle.png. Rotate from vertical to 30° above right.
const claw = `<g transform="rotate(60 64 64)">
  <path d="M39 96L22 85L13 68L15 49L26 36L36 31L34 12L40 6L49 16L56 32L61 47L70 56L79 56L89 50L96 38L96 18L106 26L117 43L121 59L117 75L105 89L85 99L82 114L67 121L47 115L43 101Z" fill="#9f2f2f" stroke="#3d0c0c" stroke-width="3" stroke-linejoin="miter"/>
  <path d="M44 98L63 105L83 99L81 113L67 119L48 113Z" fill="#661a1a"/>
  <path d="M25 48L36 40L43 46L30 56L29 69L39 82L33 86L22 73L21 61Z" fill="#b03e3e"/>
  <path d="M40 13L48 24L54 39L56 48L47 42L43 27Z" fill="#c44848"/>
  <path d="M103 30L111 45L114 60L109 73L103 78L106 59Z" fill="#922727"/>
  <path d="M39 88L57 95L78 94L95 85L87 96L78 102L63 102L47 98Z" fill="#661a1a"/>
</g>`;
const mark = `<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128" role="img" aria-label="Crawde crab claw">${claw}</svg>\n`;
const banner = `<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="320" viewBox="0 0 1280 320" role="img" aria-labelledby="title description">
  <title id="title">Crawde</title>
  <desc id="description">A little company while Claude works.</desc>
  <rect width="1280" height="320" rx="16" fill="#121416"/>
  <g transform="translate(336 82) scale(1.1)">${claw}</g>
  <text x="520" y="175" fill="#f6f3ed" font-family="Segoe UI, Arial, sans-serif" font-weight="700" font-size="76" letter-spacing="-3">Crawde</text>
  <text x="525" y="220" fill="#aaa7a1" font-family="Segoe UI, Arial, sans-serif" font-size="22">A little company while Claude works.</text>
</svg>\n`;

mkdirSync(resolve(root, 'assets'), { recursive: true });
mkdirSync(resolve(root, 'src-tauri/icons'), { recursive: true });
writeFileSync(resolve(root, 'assets/crawde-mark.svg'), mark);
writeFileSync(resolve(root, 'assets/crawde-banner.svg'), banner);
writeFileSync(resolve(root, 'assets/crawde-banner.png'), new Resvg(banner).render().asPng());
for (const size of [16, 24, 32, 128]) {
  const png = new Resvg(mark, { fitTo: { mode: 'width', value: size } }).render().asPng();
  writeFileSync(resolve(root, `src-tauri/icons/tray${size === 128 ? '' : `-${size}`}.png`), png);
}
console.log('Generated claw artwork at 128px and tray sizes, plus matching SVG/PNG banners.');
