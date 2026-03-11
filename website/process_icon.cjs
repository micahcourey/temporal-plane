const { Jimp } = require('jimp');

async function processIcon() {
    const image = await Jimp.read('public/logo.png');

    // Find bounding box of the halo by scanning the top half (y < 400) to avoid the text
    let minX = image.bitmap.width, minY = image.bitmap.height, maxX = 0;
    image.scan(0, 0, image.bitmap.width, 400, function (x, y, idx) {
        if (this.bitmap.data[idx + 3] > 0) {
            if (x < minX) minX = x;
            if (y < minY) minY = y;
            if (x > maxX) maxX = x;
        }
    });

    if (maxX >= minX) {
        const size = maxX - minX + 1;

        // Halo is circular, so height equals width. 
        // We know minY, so maxY is just minY + size
        const maxY = minY + size - 1;

        // Erase EVERYTHING outside this perfect square perfectly centered on the halo
        image.scan(0, 0, image.bitmap.width, image.bitmap.height, function (x, y, idx) {
            if (x < minX || x > maxX || y < minY || y > maxY) {
                this.bitmap.data[idx + 0] = 0;
                this.bitmap.data[idx + 1] = 0;
                this.bitmap.data[idx + 2] = 0;
                this.bitmap.data[idx + 3] = 0;
            }
        });

        // Crop exactly to this perfect bounding square
        image.crop({ x: minX, y: minY, w: size, h: size });
    }

    // Maximize the fill size
    // Scale it up significantly to 512x512 to ensure it is very prominent as a favicon
    image.resize({ w: 512, h: 512 });

    await image.write('public/icon.png');
    console.log("Processed and saved public/icon.png");
}

processIcon().catch(console.error);
