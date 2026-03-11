const { Jimp } = require('jimp');

async function processIcon() {
    const image = await Jimp.read('public/logo-source.png');

    let minX = image.bitmap.width, minY = image.bitmap.height, maxX = 0, maxY = 0;

    // Scan for bright pixels to find the brain.
    // We scan up to 534 pixels to capture the full brain but strictly avoid the top of the MNEMIX text.
    image.scan(0, 0, image.bitmap.width, 534, function (x, y, idx) {
        const r = this.bitmap.data[idx + 0];
        const g = this.bitmap.data[idx + 1];
        const b = this.bitmap.data[idx + 2];

        if (Math.max(r, g, b) > 80) {
            if (x < minX) minX = x;
            if (y < minY) minY = y;
            if (x > maxX) maxX = x;
            if (y > maxY) maxY = y;
        }
    });

    // Add padding to ensure the soft glow isn't clipped.
    // NOTE: We do NOT pad maxY beyond 534 to avoid catching the text stems.
    const padding = 20;
    minX = Math.max(0, minX - padding);
    minY = Math.max(0, minY - padding);
    maxX = Math.min(image.bitmap.width - 1, maxX + padding);
    maxY = Math.min(534, maxY + padding);

    console.log("Brain bounds with surgical padding:", minX, minY, maxX, maxY);

    if (maxX >= minX && maxY >= minY) {
        const width = maxX - minX + 1;
        const height = maxY - minY + 1;
        const size = Math.max(width, height);

        const centerX = minX + Math.floor(width / 2);
        const centerY = minY + Math.floor(height / 2);

        let cropX = centerX - Math.floor(size / 2);
        let cropY = centerY - Math.floor(size / 2);

        if (cropX < 0) cropX = 0;
        if (cropY < 0) cropY = 0;

        let finalSize = size;
        if (cropX + finalSize > image.bitmap.width) finalSize = image.bitmap.width - cropX;
        if (cropY + finalSize > image.bitmap.height) finalSize = Math.min(finalSize, image.bitmap.height - cropY);

        image.scan(0, 0, image.bitmap.width, image.bitmap.height, function (x, y, idx) {
            if (x < cropX || x >= cropX + finalSize || y < cropY || y >= cropY + finalSize || y >= 535) {
                // Completely erase outside or if y is in the text area (using 535 as a surgical cutoff)
                this.bitmap.data[idx + 0] = 0;
                this.bitmap.data[idx + 1] = 0;
                this.bitmap.data[idx + 2] = 0;
                this.bitmap.data[idx + 3] = 0;
            } else {
                const r = this.bitmap.data[idx + 0];
                const g = this.bitmap.data[idx + 1];
                const b = this.bitmap.data[idx + 2];
                let a = Math.max(r, g, b);

                if (a < 50) {
                    this.bitmap.data[idx + 3] = 0;
                } else {
                    let newA = Math.min(255, (a - 50) * 2);
                    this.bitmap.data[idx + 3] = newA;

                    if (newA > 0 && newA < 255) {
                        this.bitmap.data[idx + 0] = Math.min(255, (r * 255) / newA);
                        this.bitmap.data[idx + 1] = Math.min(255, (g * 255) / newA);
                        this.bitmap.data[idx + 2] = Math.min(255, (b * 255) / newA);
                    }
                }
            }
        });

        image.crop({ x: cropX, y: cropY, w: finalSize, h: finalSize });

        await image.write('public/logo.png');
        console.log("Saved public/logo.png");

        image.resize({ w: 512, h: 512 });
        await image.write('public/icon.png');
        console.log("Saved public/icon.png");
    } else {
        console.log("Failed to find brain.");
    }
}

processIcon().catch(console.error);
