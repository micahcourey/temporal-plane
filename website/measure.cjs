const { Jimp } = require('jimp');

async function measure() {
    const image = await Jimp.read('public/logo-source.png');

    console.log("Image size:", image.bitmap.width, "x", image.bitmap.height);

    // We expect the halo to be centered around x=320 (if width 640)
    // Let's scan y from 400 to 600 and see where the intensity is
    for (let y = 450; y < 600; y++) {
        let maxIntensity = 0;
        let countAboveThreshold = 0;
        for (let x = 0; x < image.bitmap.width; x++) {
            const idx = (image.bitmap.width * y + x) << 2;
            const r = image.bitmap.data[idx + 0];
            const g = image.bitmap.data[idx + 1];
            const b = image.bitmap.data[idx + 2];
            const intensity = Math.max(r, g, b);
            if (intensity > maxIntensity) maxIntensity = intensity;
            if (intensity > 80) countAboveThreshold++;
        }
        if (countAboveThreshold > 0) {
            console.log(`y=${y}: maxIntensity=${maxIntensity}, count=${countAboveThreshold}`);
        }
    }
}

measure().catch(console.error);
