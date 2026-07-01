import os
from PIL import Image, ImageDraw

def generate_circle_icon(color_rgb, filename):
    # Create a 32x32 image with transparent background (RGBA)
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Draw a smooth solid circle
    # bounding box: (left, top, right, bottom)
    draw.ellipse([4, 4, 27, 27], fill=color_rgb)

    # Save as .ico
    img.save(filename, format="ICO")
    print(f"Generated {filename}")

if __name__ == "__main__":
    icon_dir = os.path.join("src-tauri", "icons")
    os.makedirs(icon_dir, exist_ok=True)

    # Define colors
    colors = {
        "green.ico": (23, 201, 100, 255),    # normal state: HSL 120
        "yellow.ico": (255, 193, 7, 255),    # warning state: HSL 45
        "red.ico": (244, 67, 54, 255),       # error state: HSL 0
        "gray.ico": (158, 158, 158, 255)     # unknown state: HSL 0/0
    }

    for filename, color in colors.items():
        path = os.path.join(icon_dir, filename)
        generate_circle_icon(color, path)
