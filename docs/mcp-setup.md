# Model Context Protocol (MCP) Setup Guide

Clawvinci hosts an embedded Model Context Protocol (MCP) server at:
```
http://127.0.0.1:19789/mcp
```
This allows external AI assistants (Claude Desktop, Cursor, Claude Code, Codex, and local LLMs) to programmatically edit your timeline sequences directly alongside you.

---

## Supported Tools (53 Tools)

The embedded MCP server provides 53 specialized NLE tools, including:
- **Timeline & Media Inspection**: `get_timeline`, `list_media_items`, `probe_media_file`, `get_history_status`.
- **Clip Operations**: `split_clip`, `trim_clip`, `move_clips`, `remove_clips`, `ripple_delete`, `duplicate_clip`.
- **Track Management**: `insert_track`, `remove_track`, `reorder_tracks`, `set_track_mute`, `set_track_solo`.
- **Color & Effects**: `apply_effect`, `update_effect_param`, `apply_lut`, `adjust_color_wheels`.
- **Kinetic Text**: `insert_text_clip`, `update_text_properties`, `animate_text`.
- **Audio & Generative AI**: `analyze_audio_levels`, `detect_silence`, `generate_speech`, `generate_image`, `generate_video`.

---

## 1. Claude Desktop (Windows)

1. Open your Claude Desktop configuration file at:
   ```
   %APPDATA%\Claude\claude_desktop_config.json
   ```
2. Add the `clawvinci` MCP server configuration:
   ```json
   {
     "mcpServers": {
       "clawvinci": {
         "type": "http",
         "url": "http://127.0.0.1:19789/mcp"
       }
     }
   }
   ```
3. Restart Claude Desktop. You will see the tool hammer icon appear with Clawvinci tools.

---

## 2. Cursor IDE

1. Open Cursor Settings -> Features -> MCP Servers, or open:
   ```
   %USERPROFILE%\.cursor\mcp.json
   ```
2. Add the server entry:
   ```json
   {
     "mcpServers": {
       "clawvinci": {
         "type": "http",
         "url": "http://127.0.0.1:19789/mcp"
       }
     }
   }
   ```
3. In Cursor Composer or Chat, you can now instruct the agent to inspect and edit your active timeline directly.

---

## 3. Claude Code CLI

Run this command in any terminal:
```powershell
claude mcp add --transport http clawvinci http://127.0.0.1:19789/mcp
```

---

## 4. Codex CLI

Run this command:
```powershell
codex mcp add clawvinci --url http://127.0.0.1:19789/mcp
```
