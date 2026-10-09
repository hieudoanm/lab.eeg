import sharp from 'sharp';
import path from 'path';

const inputSvg = path.join(__dirname, '../public/og/og.svg');
const outputPng = path.join(__dirname, '../public/og/og.png');

async function generateOgImage() {
  await sharp(inputSvg, { density: 300 })
    .resize(1200, 630, { fit: 'cover' })
    .png()
    .toFile(outputPng);
  console.log(`Generated ${outputPng} (1200x630)`);
}

generateOgImage().catch(console.error);
