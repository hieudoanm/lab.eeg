import sharp from 'sharp';
import path from 'path';

const input = path.join(__dirname, '../public/icons/icon.svg');
const output = path.join(__dirname, '../public/icons/icon-180x180.png');

async function main() {
  await sharp(input, { density: 300 }).resize(180, 180).png().toFile(output);
  console.log('Generated 180x180 icon');
}

main().catch(console.error);
