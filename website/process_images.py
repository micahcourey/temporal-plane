from PIL import Image

def process():
    img = Image.open('public/logo-source.png').convert("RGBA")
    data = img.getdata()
    width, height = img.size
    
    new_data = []
    # Background is roughly RGB(7, 17, 21)
    
    # Pass 1: find bounds of non-background and non-text pixels
    min_x, min_y = width, height
    max_x, max_y = 0, 0
    
    for y in range(height):
        for x in range(width):
            r, g, b, a = data[y * width + x]
            
            # Wipe everything below y=538 to remove text
            if y >= 538:
                new_data.append((0, 0, 0, 0))
                continue
                
            # Check if pixel is dark background
            if r < 40 and g < 40 and b < 40:
                new_data.append((0, 0, 0, 0))
            else:
                new_data.append((r, g, b, 255))
                if x < min_x: min_x = x
                if y < min_y: min_y = y
                if x > max_x: max_x = x
                if y > max_y: max_y = y
                
    img.putdata(new_data)
    
    if max_x >= min_x and max_y >= min_y:
        w = max_x - min_x + 1
        h = max_y - min_y + 1
        size = max(w, h)
        center_x = min_x + w // 2
        center_y = min_y + h // 2
        
        crop_x = max(0, center_x - size // 2)
        crop_y = max(0, center_y - size // 2)
        
        # Ensure we stay in bounds
        crop_size = min(size, width - crop_x, height - crop_y)
        
        halo = img.crop((crop_x, crop_y, crop_x + crop_size, crop_y + crop_size))
        halo.save('public/logo.png')
        print("Saved perfect halo logo.png")
        
        halo_icon = halo.resize((512, 512), Image.Resampling.LANCZOS)
        halo_icon.save('public/icon.png')
        print("Saved icon.png")
        
process()
