import path from 'path';
import sharp from 'sharp';

const sizes = [72, 96, 128, 144, 152, 192, 384, 512];
const inputSvg = path.join(__dirname, '../public/icons/icon.svg');
const outputDir = path.join(__dirname, '../public/icons');

async function generateIcons() {
  for (const size of sizes) {
    const outputPath = path.join(outputDir, `icon-${size}x${size}.png`);
    await sharp(inputSvg, { density: 300 })
      .resize(size, size, { fit: 'cover' })
      .png()
      .toFile(outputPath);
    console.log(`Generated ${outputPath}`);
  }

  // Also create a favicon.ico
  const faviconPath = path.join(__dirname, '../public/favicon.ico');
  await sharp(inputSvg, { density: 300 })
    .resize(32, 32, { fit: 'cover' })
    .png()
    .toFile(faviconPath.replace('.ico', '.png'));
  console.log('Generated favicon.png');

  console.log('All icons generated successfully!');
}

generateIcons().catch(console.error);
