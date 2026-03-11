const { Jimp } = require('jimp');
async function test() {
    const icon = await Jimp.read('public/icon.png');
    const logo = await Jimp.read('public/logo.png');

    console.log("ICON - hasAlpha:", icon.hasAlpha(), " dimensions:", icon.bitmap.width, "x", icon.bitmap.height);
    console.log("LOGO - hasAlpha:", logo.hasAlpha(), " dimensions:", logo.bitmap.width, "x", logo.bitmap.height);

    // Check bottom-left pixel of logo to ensure it's transparent (checking if background and text were wiped)
    const idx = (logo.bitmap.height - 1) * logo.bitmap.width * 4;
    console.log("LOGO Bottom-Left alpha:", logo.bitmap.data[idx + 3]);

    // Check center pixel of logo to ensure it's not transparent (checking if halo exists)
    const centerX = Math.floor(logo.bitmap.width / 2);
    const centerY = Math.floor(logo.bitmap.height / 2);
    const centerIdx = (centerY * logo.bitmap.width + centerX) * 4;
    console.log("LOGO Center alpha:", logo.bitmap.data[centerIdx + 3]);
}
test().catch(console.error);
