#!/usr/bin/env python3
"""
Generates a realistic terminal recording GIF with character-by-character typing
and streaming terminal output for ShadowBox. Zero emojis, pure terminal aesthetics.
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

    # Traffic light buttons (minimal, clean)
    draw.ellipse([14, 12, 26, 24], fill=(255, 95, 86))
    draw.ellipse([34, 12, 46, 24], fill=(255, 189, 46))
    draw.ellipse([54, 12, 66, 24], fill=(39, 201, 63))

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
            bar_text = item[1]
            draw.rectangle([20, y - 2, WIDTH - 20, y + line_height], fill=BAR_BG, outline=BORDER_COLOR)
            draw.text((28, y + 2), bar_text, font=bold_font, fill=CYAN_COLOR)
            y += line_height + 8
            continue

        x = 24
        for segment in item:
            text, color = segment
            draw.text((x, y), text, font=font, fill=color)
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
        ("user@mac:~/my-app$ ", MUTED_COLOR),
    ]

    prompt_w = 24
    for t, _ in prompt:
        b = font.getbbox(t)
        prompt_w += (b[2] - b[0])

    cmd_to_type = "shadowbox claude --dangerously-skip-permissions"

    # 1. Initial idle cursor
    for i in range(2):
        frames.append(render_frame([prompt], cursor_pos=(prompt_w, 52) if i % 2 == 0 else None))
        durations.append(250)

    # 2. Typewriter: letter-by-letter typing
    typed_so_far = ""
    for char in cmd_to_type:
        typed_so_far += char
        current_line = list(prompt) + [(typed_so_far, TEXT_COLOR)]
        
        cur_w = 24
        for t, _ in current_line:
            b = font.getbbox(t)
            cur_w += (b[2] - b[0])

        frames.append(render_frame([current_line], cursor_pos=(cur_w, 52)))
        durations.append(40 if char != " " else 80)

    # 3. Enter key
    full_cmd_line = list(prompt) + [(cmd_to_type, TEXT_COLOR)]
    cur_w = 24
    for t, _ in full_cmd_line:
        b = font.getbbox(t)
        cur_w += (b[2] - b[0])

    frames.append(render_frame([full_cmd_line], cursor_pos=(cur_w, 52)))
    durations.append(350)

    # 4. Stream outputs line by line (clean text, no emojis)
    scene_lines = [full_cmd_line]

    step_lines = [
        [("shadowbox: zero-copy apfs scratchpad initialized ", CYAN_COLOR), ("(14ms, 0 B)", MUTED_COLOR)],
        [("  workspace clone: ", MUTED_COLOR), (".shadowbox/sessions/179201-984/work", TEXT_COLOR)],
        [("secret-shield: ", YELLOW_COLOR), ("3 credentials stripped from agent environment", TEXT_COLOR)],
        [("  ● blocked: AWS_SECRET_ACCESS_KEY, GITHUB_TOKEN, SSH_AUTH_SOCK", MUTED_COLOR)],
        [("  ● cloaked: .env (mocked with [SHADOWBOX_REDACTED])", MUTED_COLOR)],
        [("agent: claude-code running in sandbox...", PURPLE_COLOR)],
        [("  ● read .env -> ", MUTED_COLOR), ("denied by secret-shield", RED_COLOR)],
        [("  ● wrote src/auth/session.ts ", MUTED_COLOR), ("(+48 lines)", GREEN_COLOR)],
        [("  ● updated package.json ", MUTED_COLOR), ("(+1 line)", YELLOW_COLOR)],
        [("host repository: ", MUTED_COLOR), ("100% untouched (0 unapproved writes)", GREEN_COLOR)],
        ("__BAR__", "[c] commit changes  |  [q] discard sandbox  |  [Tab] diff viewer"),
    ]

    for line in step_lines:
        scene_lines.append(line)
        frames.append(render_frame(list(scene_lines)))
        durations.append(160)

    # 5. User pauses
    frames.append(render_frame(list(scene_lines)))
    durations.append(600)

    # 6. User presses [c]
    commit_input_line = [
        ("user@mac:~/my-app$ ", MUTED_COLOR),
        ("keystroke [c] -> commit approved", CYAN_COLOR),
    ]
    scene_lines.append(commit_input_line)
    frames.append(render_frame(list(scene_lines)))
    durations.append(250)

    # 7. Final confirmation
    final_line = [
        ("shadowbox: changes applied to host repository. session cleaned.", GREEN_COLOR)
    ]
    scene_lines.append(final_line)
    frames.append(render_frame(list(scene_lines)))
    durations.append(3500)

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
    print(f"Generated {output_path} ({len(frames)} frames)")

if __name__ == "__main__":
    main()
