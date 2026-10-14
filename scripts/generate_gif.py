#!/usr/bin/env python3
"""
Generates a realistic terminal recording GIF with character-by-character typing
and streaming terminal output for ShadowBox, styled cleanly like AGGIB/Stroq.
"""

import os
from PIL import Image, ImageDraw, ImageFont

WIDTH = 760
HEIGHT = 440
BG_COLOR = (13, 17, 23)          # GitHub Dark Dimmed #0d1117
TITLE_BG = (22, 27, 34)          # #161b22
BORDER_COLOR = (48, 54, 61)      # #30363d
TEXT_COLOR = (201, 209, 217)     # Default light gray #c9d1d9
MUTED_COLOR = (139, 148, 158)    # Gray #8b949e
GREEN_COLOR = (63, 185, 80)      # #3fb950
YELLOW_COLOR = (210, 153, 34)    # #d29922
CYAN_COLOR = (88, 166, 255)      # #58a6ff
RED_COLOR = (248, 81, 73)        # #f85149
PURPLE_COLOR = (188, 140, 255)   # #bc8cff
BAR_BG = (27, 34, 44)

# Load font
try:
    font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 13)
    bold_font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 13)
    header_font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 11)
except Exception:
    font = ImageFont.load_default()
    bold_font = font
    header_font = font

def create_base_window():
    img = Image.new("RGB", (WIDTH, HEIGHT), BG_COLOR)
    draw = ImageDraw.Draw(img)

    # Window border & titlebar
    draw.rectangle([0, 0, WIDTH - 1, 36], fill=TITLE_BG)
    draw.line([0, 36, WIDTH - 1, 36], fill=BORDER_COLOR, width=1)
    draw.rectangle([0, 0, WIDTH - 1, HEIGHT - 1], outline=BORDER_COLOR, width=1)

    # Traffic light buttons
    draw.ellipse([14, 12, 26, 24], fill=(255, 95, 86))    # Close red
    draw.ellipse([34, 12, 46, 24], fill=(255, 189, 46))   # Minimize yellow
    draw.ellipse([54, 12, 66, 24], fill=(39, 201, 63))    # Zoom green

    # Titlebar text
    title = "shadowbox — terminal"
    draw.text((WIDTH // 2 - 60, 11), title, font=header_font, fill=MUTED_COLOR)

    return img

def render_frame(lines, cursor_pos=None):
    img = create_base_window()
    draw = ImageDraw.Draw(img)

    y = 52
    line_height = 20

    for item in lines:
        if isinstance(item, tuple) and len(item) == 2 and item[0] == "__BAR__":
            # Render a control banner bar
            bar_text = item[1]
            draw.rectangle([20, y - 2, WIDTH - 20, y + line_height], fill=BAR_BG, outline=BORDER_COLOR)
            draw.text((28, y + 2), bar_text, font=bold_font, fill=CYAN_COLOR)
            y += line_height + 8
            continue

        x = 24
        # item is a list of (text, color) tuples
        for segment in item:
            text, color = segment
            draw.text((x, y), text, font=font, fill=color)
            # measure width
            bbox = font.getbbox(text)
            x += (bbox[2] - bbox[0])

        y += line_height

    if cursor_pos:
        cx, cy = cursor_pos
        draw.rectangle([cx, cy, cx + 8, cy + 15], fill=CYAN_COLOR)

    return img

def main():
    frames = []
    durations = []

    prompt = [
        ("➜ ", GREEN_COLOR),
        ("my-project ", CYAN_COLOR),
        ("git:(", MUTED_COLOR),
        ("main", YELLOW_COLOR),
        (")", MUTED_COLOR),
        (" ", TEXT_COLOR),
    ]

    # Calculate prompt width
    prompt_w = 24
    for t, _ in prompt:
        b = font.getbbox(t)
        prompt_w += (b[2] - b[0])

    cmd_to_type = "shadowbox claude --dangerously-skip-permissions"

    # 1. Initial idle frames with blinking cursor
    for i in range(3):
        frames.append(render_frame([prompt], cursor_pos=(prompt_w, 52) if i % 2 == 0 else None))
        durations.append(250)

    # 2. Typewriter: letter-by-letter typing
    typed_so_far = ""
    for char in cmd_to_type:
        typed_so_far += char
        current_line = list(prompt) + [(typed_so_far, TEXT_COLOR)]
        
        # calculate cursor x
        cur_w = 24
        for t, _ in current_line:
            b = font.getbbox(t)
            cur_w += (b[2] - b[0])

        frames.append(render_frame([current_line], cursor_pos=(cur_w, 52)))
        durations.append(45 if char != " " else 90)

    # 3. Pause before Enter
    full_cmd_line = list(prompt) + [(cmd_to_type, TEXT_COLOR)]
    cur_w = 24
    for t, _ in full_cmd_line:
        b = font.getbbox(t)
        cur_w += (b[2] - b[0])

    frames.append(render_frame([full_cmd_line], cursor_pos=(cur_w, 52)))
    durations.append(400)

    # 4. Stream outputs line by line
    scene_lines = [full_cmd_line]

    step_lines = [
        [("🛡️  ShadowBox: Zero-Copy APFS scratchpad initialized ", CYAN_COLOR), ("(14ms, 0 B)", MUTED_COLOR)],
        [("   Workspace clone: ", MUTED_COLOR), (".shadowbox/sessions/179201-984/work", TEXT_COLOR)],
        [("🔒 SecretShield: ", YELLOW_COLOR), ("3 credentials stripped from agent environment", TEXT_COLOR)],
        [("   • Blocked: AWS_SECRET_ACCESS_KEY, GITHUB_TOKEN, SSH_AUTH_SOCK", MUTED_COLOR)],
        [("   • Cloaked: .env (mocked with [SHADOWBOX_REDACTED])", MUTED_COLOR)],
        [("⚡ Claude Code running in isolated sandbox...", PURPLE_COLOR)],
        [("   [Claude] Reading .env... ", MUTED_COLOR), ("-> [BLOCKED by SecretShield]", RED_COLOR)],
        [("   [Claude] Writing src/auth/session.ts ", MUTED_COLOR), ("(+48 lines)", GREEN_COLOR)],
        [("   [Claude] Updating package.json ", MUTED_COLOR), ("(+1 line)", YELLOW_COLOR)],
        [("📁 Real Host Repository: ", MUTED_COLOR), ("100% UNTOUCHED (0 unapproved writes)", GREEN_COLOR)],
        ("__BAR__", "[c] Commit approved changes  |  [q] Discard sandbox  |  [Tab] View Diffs"),
    ]

    for line in step_lines:
        scene_lines.append(line)
        frames.append(render_frame(list(scene_lines)))
        durations.append(180)

    # 5. User pauses to inspect
    frames.append(render_frame(list(scene_lines)))
    durations.append(700)

    # 6. User presses [c]
    commit_input_line = [
        ("➜ ", GREEN_COLOR),
        ("User pressed ", MUTED_COLOR),
        ("[c]", GREEN_COLOR),
        (" -> Commit approved.", CYAN_COLOR),
    ]
    scene_lines.append(commit_input_line)
    frames.append(render_frame(list(scene_lines)))
    durations.append(250)

    # 7. Atomically committed to host
    final_line = [
        ("✅ Changes atomically applied to host workspace. Session 179201 cleaned.", GREEN_COLOR)
    ]
    scene_lines.append(final_line)
    frames.append(render_frame(list(scene_lines)))
    durations.append(3500) # Hold final frame for 3.5s before looping

    output_path = "docs/assets/case-study.gif"
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    frames[0].save(
        output_path,
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
    )
    print(f"✅ Generated {output_path} ({len(frames)} frames)")

if __name__ == "__main__":
    main()
