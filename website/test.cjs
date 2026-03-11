const { Jimp } = require('jimp');
async function test() {
    const image = await Jimp.read('public/logo.png');
    // Top-left pixel is usually the background color
    const idx = 0;
    console.log(
        "Top-left px rgb:",
        image.bitmap.data[idx + 0],
        image.bitmap.data[idx + 1],
        image.bitmap.data[idx + 2]
    );

    const idx2 = (600 * 640 + 10) * 4;
    console.log(
        "Bottom-left px rgb:",
        image.bitmap.data[idx2 + 0],
        image.bitmap.data[idx2 + 1],
        image.bitmap.data[idx2 + 2]
    );
}
test();
