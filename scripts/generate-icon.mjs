import sharp from "sharp";
import pngToIco from "png-to-ico";
import { writeFileSync } from "fs";

const SIZE = 512;
const HALF = SIZE / 2;

// Speaker icon with flip arrows - SVG design
const svg = `
<svg width="${SIZE}" height="${SIZE}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#3b82f6"/>
      <stop offset="100%" style="stop-color:#1d4ed8"/>
    </linearGradient>
  </defs>

  <!-- Rounded square background -->
  <rect x="32" y="32" width="448" height="448" rx="96" ry="96" fill="url(#bg)"/>

  <!-- Speaker body -->
  <rect x="140" y="200" width="60" height="112" rx="8" fill="white"/>

  <!-- Speaker cone -->
  <polygon points="200,175 290,200 290,312 200,337" fill="white"/>

  <!-- Sound wave arcs -->
  <path d="M 310 210 Q 360 256 310 302" fill="none" stroke="white" stroke-width="20" stroke-linecap="round"/>
  <path d="M 340 178 Q 410 256 340 334" fill="none" stroke="white" stroke-width="20" stroke-linecap="round"/>

  <!-- Flip arrow (circular) at bottom right -->
  <path d="M 370 390 A 35 35 0 1 1 400 360" fill="none" stroke="white" stroke-width="14" stroke-linecap="round"/>
  <polygon points="405,374 418,356 392,358" fill="white"/>
</svg>
`;

const iconDir = new URL("../src-tauri/icons/", import.meta.url);

const pngBuffer = await sharp(Buffer.from(svg)).png().toBuffer();

// Generate multiple sizes
const sizes = [32, 128, 256, 512];
for (const s of sizes) {
  const resized = await sharp(pngBuffer).resize(s, s).png().toBuffer();
  const name =
    s === 512
      ? "icon.png"
      : s === 256
        ? "128x128@2x.png"
        : `${s}x${s}.png`;
  writeFileSync(new URL(name, iconDir), resized);
}

// Generate .ico (256px for Windows)
const ico256 = await sharp(pngBuffer).resize(256, 256).png().toBuffer();
const icoBuffer = await pngToIco(ico256);
writeFileSync(new URL("icon.ico", iconDir), icoBuffer);

// Generate .icns placeholder (just copy 512 png, real icns needs iconutil)
writeFileSync(
  new URL("icon.icns", iconDir),
  await sharp(pngBuffer).resize(512, 512).png().toBuffer(),
);

console.log("Icons generated:");
sizes.forEach((s) => {
  const name =
    s === 512
      ? "icon.png"
      : s === 256
        ? "128x128@2x.png"
        : `${s}x${s}.png`;
  console.log(`  ${name} (${s}x${s})`);
});
console.log("  icon.ico (256x256)");
console.log("  icon.icns (512x512)");
