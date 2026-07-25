#!/usr/bin/env python3
"""Generate Tywindb animation frames for video creation."""

from PIL import Image, ImageDraw, ImageFont
import math
import os

# Settings
WIDTH = 1280
HEIGHT = 720
FPS = 30
DURATION = 8
TOTAL_FRAMES = FPS * DURATION
OUTPUT_DIR = "/home/paong/tywindb/assets/frames"

# Colors
BLACK = (0, 0, 0)
ORANGE = (255, 107, 53)
WHITE = (255, 255, 255)
GRAY = (107, 114, 128)

def ease_out_back(t):
    c1 = 1.70158
    c3 = c1 + 1
    return 1 + c3 * pow(t - 1, 3) + c1 * pow(t - 1, 2)

def draw_rounded_rect(draw, bbox, radius, fill):
    x0, y0, x1, y1 = bbox
    draw.rectangle([x0+radius, y0, x1-radius, y1], fill=fill)
    draw.rectangle([x0, y0+radius, x1, y1-radius], fill=fill)
    draw.pieslice([x0, y0, x0+2*radius, y0+2*radius], 180, 270, fill=fill)
    draw.pieslice([x1-2*radius, y0, x1, y0+2*radius], 270, 360, fill=fill)
    draw.pieslice([x0, y1-2*radius, x0+2*radius, y1], 90, 180, fill=fill)
    draw.pieslice([x1-2*radius, y1-2*radius, x1, y1], 0, 90, fill=fill)

def create_frame(frame_num):
    img = Image.new('RGB', (WIDTH, HEIGHT), BLACK)
    draw = ImageDraw.Draw(img)
    
    # Grid background
    for x in range(0, WIDTH, 40):
        draw.line([(x, 0), (x, HEIGHT)], fill=(40, 40, 40), width=1)
    for y in range(0, HEIGHT, 40):
        draw.line([(0, y), (WIDTH, y)], fill=(40, 40, 40), width=1)
    
    progress = frame_num / TOTAL_FRAMES
    
    # Animation phases
    logo_scale = 1.0
    logo_alpha = 1.0
    text_alpha = 0.0
    text_y_offset = 20
    badge_alpha = 0.0
    fade_alpha = 1.0
    
    # Logo appear (0-2s)
    if progress < 0.25:
        t = progress / 0.25
        logo_scale = ease_out_back(t)
        logo_alpha = min(1.0, t * 2)
    
    # Text appear (2-4s)
    if 0.25 <= progress < 0.5:
        t = (progress - 0.25) / 0.25
        text_alpha = min(1.0, t * 2)
        text_y_offset = 20 * (1 - t)
    elif progress >= 0.5:
        text_alpha = 1.0
        text_y_offset = 0
    
    # Badge appear (4-5s)
    if 0.5 <= progress < 0.625:
        t = (progress - 0.5) / 0.125
        badge_alpha = min(1.0, t * 2)
    elif progress >= 0.625:
        badge_alpha = 1.0
    
    # Fade out (7-8s)
    if progress > 0.875:
        t = (progress - 0.875) / 0.125
        fade_alpha = 1 - t
    
    # Draw logo
    logo_size = int(120 * logo_scale)
    logo_x = WIDTH // 2 - logo_size // 2
    logo_y = HEIGHT // 2 - logo_size // 2 - 40
    
    if logo_alpha > 0:
        # Glow
        glow_size = int(180 * logo_scale)
        glow_x = WIDTH // 2 - glow_size // 2
        glow_y = HEIGHT // 2 - glow_size // 2 - 40
        glow_img = Image.new('RGBA', (WIDTH, HEIGHT), (0, 0, 0, 0))
        glow_draw = ImageDraw.Draw(glow_img)
        glow_alpha_val = int(60 * logo_alpha * (0.7 + 0.3 * math.sin(progress * 8)))
        glow_draw.ellipse(
            [glow_x, glow_y, glow_x + glow_size, glow_y + glow_size],
            fill=(255, 107, 53, glow_alpha_val)
        )
        img.paste(Image.alpha_composite(img.convert('RGBA'), glow_img).convert('RGB'))
        draw = ImageDraw.Draw(img)
        
        # Orange circle
        draw.ellipse(
            [logo_x, logo_y, logo_x + logo_size, logo_y + logo_size],
            fill=ORANGE
        )
        
        # White database shape
        cx = logo_x + logo_size // 2
        cy = logo_y + logo_size // 2
        inner = int(logo_size * 0.22)
        ellipse_h = int(logo_size * 0.08)
        
        # Top ellipse
        draw.ellipse(
            [cx - inner, cy - int(logo_size * 0.15) - ellipse_h,
             cx + inner, cy - int(logo_size * 0.15) + ellipse_h],
            fill=WHITE
        )
        # Rectangle
        draw.rectangle(
            [cx - inner, cy - int(logo_size * 0.15),
             cx + inner, cy + int(logo_size * 0.15)],
            fill=WHITE
        )
        # Bottom ellipse
        draw.ellipse(
            [cx - inner, cy + int(logo_size * 0.15) - ellipse_h,
             cx + inner, cy + int(logo_size * 0.15) + ellipse_h],
            fill=WHITE
        )
        
        # Lightning bolt
        bolt_w = max(2, int(logo_size * 0.06))
        bolt = [
            (cx + int(logo_size * 0.04), cy - int(logo_size * 0.18)),
            (cx - int(logo_size * 0.06), cy),
            (cx + int(logo_size * 0.02), cy),
            (cx - int(logo_size * 0.04), cy + int(logo_size * 0.18)),
        ]
        draw.line(bolt, fill=ORANGE, width=bolt_w)
    
    # Text
    if text_alpha > 0:
        try:
            font_large = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 56)
            font_small = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 16)
        except:
            font_large = ImageFont.load_default()
            font_small = ImageFont.load_default()
        
        text_y = logo_y + logo_size + 30 + int(text_y_offset)
        
        # "Ty" white, "windb" orange
        ty_bbox = draw.textbbox((0, 0), "Ty", font=font_large)
        ty_w = ty_bbox[2] - ty_bbox[0]
        windb_bbox = draw.textbbox((0, 0), "windb", font=font_large)
        windb_w = windb_bbox[2] - windb_bbox[0]
        total_w = ty_w + windb_w
        start_x = WIDTH // 2 - total_w // 2
        
        alpha_val = int(255 * text_alpha)
        draw.text((start_x, text_y), "Ty", fill=(255, 255, 255, alpha_val), font=font_large)
        draw.text((start_x + ty_w, text_y), "windb", fill=(255, 107, 53, alpha_val), font=font_large)
        
        # Tagline
        tagline = "Fast. Simple. Secure."
        tag_bbox = draw.textbbox((0, 0), tagline, font=font_small)
        tag_w = tag_bbox[2] - tag_bbox[0]
        draw.text((WIDTH // 2 - tag_w // 2, text_y + 70), tagline, 
                  fill=(107, 114, 128, alpha_val), font=font_small)
    
    # Version badge
    if badge_alpha > 0:
        try:
            font_badge = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 13)
        except:
            font_badge = ImageFont.load_default()
        
        badge_text = "Version 0.97"
        bb = draw.textbbox((0, 0), badge_text, font=font_badge)
        bw = bb[2] - bb[0] + 32
        bh = 28
        bx = WIDTH // 2 - bw // 2
        by = HEIGHT // 2 + 110
        
        badge_img = Image.new('RGBA', (WIDTH, HEIGHT), (0, 0, 0, 0))
        badge_draw = ImageDraw.Draw(badge_img)
        badge_draw.rounded_rectangle(
            [bx, by, bx + bw, by + bh],
            radius=14,
            fill=(255, 107, 53, int(30 * badge_alpha)),
            outline=(255, 107, 53, int(100 * badge_alpha))
        )
        badge_draw.text((bx + 16, by + 6), badge_text, 
                       fill=(255, 107, 53, int(255 * badge_alpha)), font=font_badge)
        img = Image.alpha_composite(img.convert('RGBA'), badge_img).convert('RGB')
    
    # Fade out
    if fade_alpha < 1:
        overlay = Image.new('RGB', (WIDTH, HEIGHT), BLACK)
        img = Image.blend(img, overlay, 1 - fade_alpha)
    
    return img

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    print(f"Generating {TOTAL_FRAMES} frames...")
    
    for i in range(TOTAL_FRAMES):
        img = create_frame(i)
        img.save(os.path.join(OUTPUT_DIR, f"frame_{i:04d}.png"))
        if (i + 1) % 30 == 0:
            print(f"  {i + 1}/{TOTAL_FRAMES}")
    
    print(f"Done! Frames: {OUTPUT_DIR}")
    print(f"\nCreate video:")
    print(f"ffmpeg -framerate {FPS} -i {OUTPUT_DIR}/frame_%04d.png -c:v libx264 -pix_fmt yuv420p /home/paong/tywindb/assets/tywindb-intro.mp4")

if __name__ == "__main__":
    main()
