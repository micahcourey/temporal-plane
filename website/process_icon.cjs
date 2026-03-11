const { Jimp } = require('jimp');

async function processIcon() {
    const image = await Jimp.read('public/logo.png');

    // The original image is 640x640 with text at the bottom. Crop to the top 450 pixels.
    image.crop({ x: 0, y: 0, w: 640, h: 450 });

    // Convert dark background to transparent
    image.scan(0, 0, image.bitmap.width, image.bitmap.height, function (x, y, idx) {
        const r = this.bitmap.data[idx + 0];
        const g = this.bitmap.data[idx + 1];
        const b = this.bitmap.data[idx + 2];

        // Estimate alpha from brightness
        let a = Math.max(r, g, b);

        // Boost alpha a bit so mid-tones are strongly opaque
        a = Math.min(255, a * 1.5);

        // Any dark background colors below a threshold should be 100% transparent.
        if (a < 40) {
            a = 0;
        }

        // Adjust colors to account for new alpha transparency (un-premultiply)
        if (a > 0 && a < 255) {
            this.bitmap.data[idx + 0] = Math.min(255, (r * 255) / a);
            this.bitmap.data[idx + 1] = Math.min(255, (g * 255) / a);
            this.bitmap.data[idx + 2] = Math.min(255, (b * 255) / a);
        }

        this.bitmap.data[idx + 3] = Math.floor(a);
    });

    // Find bounding box of non-transparent pixels to completely bound the image.
    let minX = image.bitmap.width, minY = image.bitmap.height, maxX = 0, maxY = 0;
    image.scan(0, 0, image.bitmap.width, image.bitmap.height, function (x, y, idx) {
        if (this.bitmap.data[idx + 3] > 0) {
            if (x < minX) minX = x;
            if (y < minY) minY = y;
            if (x > maxX) maxX = x;
            if (y > maxY) maxY = y;
        }
    });

    if (maxX >= minX && maxY >= minY) {
        const w = maxX - minX + 1;
        const h = maxY - minY + 1;
        // Crop exactly to the non-transparent content
        image.crop({ x: minX, y: minY, w: w, h: h });
    }

    // Maximize the fill size
    // Scale it up significantly to 512x512 to ensure it is very prominent as a favicon
    image.resize({ w: 512, h: 512 });

    await image.write('public/icon.png');
    console.log("Processed and saved public/icon.png");
}

processIcon().catch(console.error);
