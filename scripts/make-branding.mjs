import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Resvg } from '@resvg/resvg-js';

const root = fileURLToPath(new URL('../', import.meta.url));
const claw = `
  <path d="M44 96L45 113Q64 121 83 113L84 96C106 89 118 72 116 51C115 37 107 26 98 21L97 43C96 55 89 63 79 64L65 55C57 48 54 37 52 25C51 17 47 10 42 9C39 8 36 11 37 15L37 33C25 39 18 49 17 62C16 78 26 92 44 96Z" fill="#f18b60" stroke="#6d3428" stroke-width="3" stroke-linejoin="round"/>
  <path d="M45 98Q64 106 83 98L82 112Q64 119 46 112Z" fill="#cf6846"/>
  <path d="M37 47C29 52 25 59 25 67C25 75 30 82 37 86" fill="none" stroke="#ffc4a0" stroke-width="6" stroke-linecap="round"/>
`;
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
