#!/usr/bin/env python3
"""Create video from animation frames."""

import imageio
import os

INPUT_DIR = "/home/paong/tywindb/assets/frames"
OUTPUT_FILE = "/home/paong/tywindb/assets/tywindb-intro.mp4"
FPS = 30

def main():
    frames = []
    frame_files = sorted([f for f in os.listdir(INPUT_DIR) if f.endswith('.png')])
    
    print(f"Loading {len(frame_files)} frames...")
    for f in frame_files:
        img = imageio.imread(os.path.join(INPUT_DIR, f))
        frames.append(img)
    
    print(f"Creating video at {FPS} FPS...")
    imageio.mimwrite(OUTPUT_FILE, frames, fps=FPS, codec='libx264', quality=8)
    
    size_mb = os.path.getsize(OUTPUT_FILE) / (1024 * 1024)
    print(f"Done! Video: {OUTPUT_FILE} ({size_mb:.2f} MB)")

if __name__ == "__main__":
    main()
